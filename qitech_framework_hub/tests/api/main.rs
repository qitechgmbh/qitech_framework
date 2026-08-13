use std::borrow::Cow;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::bail;
use chrono::Utc;
use qitech_framework_core::LogLevel;
use qitech_framework_core::LogOrigin;
use qitech_framework_core::LogRecord;
use qitech_framework_core::MachineConfigMutation;
use qitech_framework_core::ident::MachineIdentification;
use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::MachineMeasurement;
use qitech_framework_core::MachineMeasurementVec;
use qitech_framework_core::MachineStateMutation;
use qitech_framework_core::schema::MachinesReport;
use qitech_framework_core::OperationOrigin;
use qitech_framework_core::OperationResult;
use qitech_framework_core::RuntimeEvent;
use qitech_framework_core::RuntimeInitEvent;
use qitech_framework_core::RuntimeReport;
use qitech_framework_core::RuntimeReportData;
use qitech_framework_core::ScalarValue;
use qitech_framework_core::vendors;
use qitech_framework_hub::Config;
use qitech_framework_hub::DatabaseConfig;
use qitech_framework_hub::Embedded;
use testcontainers::GenericImage;
use testcontainers::ImageExt;
use testcontainers::core::ContainerPort;
use testcontainers::core::Mount;
use testcontainers::core::WaitFor;
use testcontainers::core::wait::HttpWaitStrategy;
use testcontainers::runners::AsyncRunner;
use tokio::process::Command;
use tokio::time::sleep;

pub const CLICKHOUSE_PORT: ContainerPort = ContainerPort::Tcp(8123);

#[tokio::test]
async fn my_test() -> anyhow::Result<()> {
    let mut config_path = env!("CARGO_MANIFEST_DIR").to_string();
    config_path.push_str("/tests/api/clickhouse/users.xml");

    // --- initialize database ---
    let container = GenericImage::new("clickhouse/clickhouse-server", "26.6")
        .with_wait_for(WaitFor::http(
            HttpWaitStrategy::new("/")
                .with_port(CLICKHOUSE_PORT)
                .with_expected_status_code(200_u16),
        ))
        .with_mount(Mount::bind_mount(
            config_path,
            "/etc/clickhouse-server/users.d/custom.xml",
        ))
        .start()
        .await?;

    let host = container.get_host().await?;
    let port = container.get_host_port_ipv4(CLICKHOUSE_PORT).await?;
    let url = format!("http://{}:{}", host, port);

    // --- initialize hub ---
    let config = Config {
        db: DatabaseConfig {
            url,
            name: "control_hub".into(),
            user: "default".into(),
            password: None,
        },
        auto_migrate: true,
        commit_interval: Duration::from_millis(2500),
        api_port: 3000,
    };

    // give it some time to initialize
    sleep(Duration::from_millis(100)).await;

    // --- create hub instance ---
    let (hub, mut session) = Embedded::new(
        config,
        vec![
            include_str!("../schemas/machine_0.yaml").to_string(),
            include_str!("../schemas/machine_1.yaml").to_string(),
        ],
    )
    .await?;

    tokio::spawn(hub.run());

    // give hub time to initialize
    sleep(Duration::from_millis(250)).await;

    // --- Step 1: emit connected machines ---
    let ident_m0 = MachineIdentificationUnique {
        identification: MachineIdentification {
            vendor_id: vendors::QITECH.id,
            machine_id: 10,
        },
        serial: 0,
    };

    let ident_m1 = MachineIdentificationUnique {
        identification: MachineIdentification {
            vendor_id: vendors::QITECH.id,
            machine_id: 20,
        },
        serial: 1,
    };

    let now = Utc::now();
    session.export(RuntimeReport {
        runtime: RuntimeReportData {
            events: vec![
                RuntimeEvent {
                    timestamp: now,
                    kind: RuntimeInitEvent::MachineConnected { ident: ident_m1 },
                },
                RuntimeEvent {
                    timestamp: now,
                    kind: RuntimeInitEvent::MachineConnected { ident: ident_m1 },
                },
            ],
            ..Default::default()
        },
        ..Default::default()
    });

    // --- Step 2, emit diconnected machine ---
    sleep(Duration::from_millis(20)).await;

    session.export(RuntimeReport {
        runtime: RuntimeReportData {
            events: vec![RuntimeEvent {
                timestamp: Utc::now(),
                kind: RuntimeInitEvent::MachineDisconnected { ident: ident_m0 },
            }],
            ..Default::default()
        },
        ..Default::default()
    });

    // --- Step 3: Export some data ---
    let machine = ident_m1;

    let logs = vec![LogRecord {
        timestamp: Utc::now(),
        level: LogLevel::Debug,
        origin: LogOrigin::Machine(machine),
        message: "Hello World".to_string(),
        attributes: Default::default(),
    }];

    let config_mutations = vec![MachineConfigMutation {
        timestamp: Utc::now(),
        machine,
        path: Cow::Borrowed("temperature.target"),
        value: ScalarValue::Float(Some(22.0)),
        origin: OperationOrigin::Request { request_id: 0 },
        result: OperationResult::Success,
    }];

    let state_mutations = vec![
        MachineStateMutation {
            timestamp: Utc::now(),
            machine,
            path: Cow::Borrowed("heating"),
            value: ScalarValue::Boolean(Some(true)),
        },
        MachineStateMutation {
            timestamp: Utc::now(),
            machine,
            path: Cow::Borrowed("cooling"),
            value: ScalarValue::Boolean(Some(true)),
        },
    ];

    let mut measurements = MachineMeasurementVec::new();
    measurements.push(MachineMeasurement {
        machine,
        path: Cow::Borrowed("temperature.current"),
        value: Some(9.0),
    });

    session.export(RuntimeReport {
        machines: MachinesReport {
            config_mutations,
            state_mutations,
            measurements,
            ..Default::default()
        },
        logs,
        ..Default::default()
    });

    // --- Step 4: run api requests ---

    // println!("running api requests");
    // run_bruno_requests().await?;

    Ok(())
}

async fn run_bruno_requests() -> anyhow::Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/api/requests");

    let output = Command::new("bru")
        .current_dir(root)
        .arg("run")
        .output()
        .await?;

    println!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    println!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));

    if !output.status.success() {
        bail!("Bruno execution failed");
    }

    Ok(())
}
