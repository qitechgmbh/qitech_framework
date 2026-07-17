use anyhow::bail;
use chrono::Utc;
use control_core::{
    LogLevel, LogOrigin, LogRecord, MachineConfigMutation, 
    MachineIdentificationUnique, MachineMeasurement, MachineStateMutation, 
    MachinesReport, OperationResult, Origin, RuntimeEvent, RuntimeEventKind, 
    RuntimeReport, RuntimeReportData, ScalarValue, vendors, MachineMeasurementVec,
};
use control_hub::{Config, Embedded, DatabaseConfig};
use std::{borrow::Cow, path::PathBuf, time::Duration};
use tokio::{process::Command, time::sleep};

use testcontainers::{
    GenericImage, ImageExt, core::{ContainerPort, Mount, WaitFor, wait::HttpWaitStrategy}, runners::AsyncRunner,
};

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
    let (app, mut session) = Embedded::init(
        config,
        vec![
            include_str!("../schemas/machine_0.yaml").to_string(),
            include_str!("../schemas/machine_1.yaml").to_string(),
        ],
    )
    .await?;

    tokio::spawn(app.run());

    // give hub time to initialize
    sleep(Duration::from_millis(250)).await;

    // --- Step 1: emit connected machines ---
    let ident_m0 = MachineIdentificationUnique {
        vendor: vendors::QITECH.id,
        machine: 10,
        serial: 0,
    };

    let ident_m1 = MachineIdentificationUnique {
        vendor: vendors::QITECH.id,
        machine: 20,
        serial: 1,
    };

    let now = Utc::now();
    session.export(RuntimeReport {
        runtime: RuntimeReportData {
            events: vec![
                RuntimeEvent {
                    timestamp: now,
                    kind: RuntimeEventKind::MachineConnected { ident: ident_m1 },
                },
                RuntimeEvent {
                    timestamp: now,
                    kind: RuntimeEventKind::MachineConnected { ident: ident_m1 },
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
            events: vec![
                RuntimeEvent {
                    timestamp: Utc::now(),
                    kind: RuntimeEventKind::MachineDisconnected { ident: ident_m0 },
                },
            ],
            ..Default::default()
        },
        ..Default::default()
    });

    // --- Step 3: Export some data ---
    let ident = ident_m1;

    let logs = vec![LogRecord {
        timestamp: Utc::now(),
        level: LogLevel::Debug,
        origin: LogOrigin::Machine(ident),
        message: "Hello World".to_string(),
        attributes: Default::default(),
    }];

    let config_mutations = vec![MachineConfigMutation {
        timestamp: Utc::now(),
        ident,
        name: Cow::Borrowed("temperature.target"),
        value: ScalarValue::Float { value: Some(22.0) },
        origin: Origin::Request { request_id: 0 },
        result: OperationResult::Success,
    }];

    let state_mutations = vec![
        MachineStateMutation {
            timestamp: Utc::now(),
            ident,
            name: Cow::Borrowed("heating"),
            value: ScalarValue::Boolean { value: Some(true) },
        },
        MachineStateMutation {
            timestamp: Utc::now(),
            ident,
            name: Cow::Borrowed("cooling"),
            value: ScalarValue::Boolean { value: Some(true) },
        },
    ];

    let mut measurements = MachineMeasurementVec::new();
    measurements.push(MachineMeasurement {
        ident,
        name: "temperature.current".to_string(),
        value: 9.0,
        null: false,
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
