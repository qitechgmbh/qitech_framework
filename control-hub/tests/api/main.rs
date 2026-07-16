use anyhow::bail;
use chrono::Utc;
use control_core::{
    ConfigMutationOrigin, ConfigMutationRecord, ConfigMutationResult, LogLevel, LogOrigin,
    LogRecord, MachineIdentificationUnique, MeasurementSnapshot, MeasurementSnapshotVec,
    RuntimeEvent, RuntimeEventKind, RuntimeExport, ScalarValue, StateMutationRecord, vendors,
};
use control_hub::{Config, ControlHub, DatabaseConfig};
use std::{borrow::Cow, path::PathBuf, time::Duration};
use tokio::{process::Command, sync::watch, time::sleep};

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

    // --- migrate database ---
    control_hub::migrate(&url).await?;

    // --- initialize hub ---
    let config = Config {
        database: DatabaseConfig {
            url: url,
            name: "control_hub".into(),
            user: "default".into(),
            password: None,
        },
        export_interval: Duration::from_millis(2500),
        api_address: "0.0.0.0:3000".into(),
    };

    let (shutdown_tx, shutdown_rx) = watch::channel(());
    _ = shutdown_tx;

    // give it some time to initialize
    sleep(Duration::from_millis(100)).await;

    // --- create hub instance ---
    let (app, mut session) = ControlHub::init_embedded(
        config,
        vec![
            include_str!("schemas/machine_0.yaml").to_string(),
            include_str!("schemas/machine_1.yaml").to_string(),
        ],
        shutdown_rx,
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
    session.export(RuntimeExport {
        runtime_events: vec![
            RuntimeEvent {
                timestamp: now,
                kind: RuntimeEventKind::MachineConnected(ident_m0),
            },
            RuntimeEvent {
                timestamp: now,
                kind: RuntimeEventKind::MachineConnected(ident_m1),
            },
        ],
        ..Default::default()
    });

    // --- Step 2, emit diconnected machine ---
    sleep(Duration::from_millis(20)).await;

    session.export(RuntimeExport {
        runtime_events: vec![RuntimeEvent {
            timestamp: Utc::now(),
            kind: RuntimeEventKind::MachineDisconnected(ident_m0),
        }],
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

    let config_mutations = vec![ConfigMutationRecord {
        timestamp: Utc::now(),
        ident,
        name: Cow::Borrowed("temperature.target"),
        value: ScalarValue::Float(Some(22.0)),
        origin: ConfigMutationOrigin::User { request_id: 0 },
        result: ConfigMutationResult::Success,
    }];

    let state_mutations = vec![
        StateMutationRecord {
            timestamp: Utc::now(),
            ident,
            name: Cow::Borrowed("heating"),
            value: ScalarValue::Boolean(Some(true)),
        },
        StateMutationRecord {
            timestamp: Utc::now(),
            ident,
            name: Cow::Borrowed("cooling"),
            value: ScalarValue::Boolean(Some(true)),
        },
    ];

    let mut machine_measurements = MeasurementSnapshotVec::new();
    machine_measurements.push(MeasurementSnapshot {
        ident,
        name: "temperature.current".to_string(),
        value: 9.0,
        null: false,
    });

    session.export(RuntimeExport {
        created_at: Utc::now(),
        logs,
        config_mutations,
        state_mutations,
        machine_measurements,
        ..Default::default()
    });

    // --- Step 4: run api requests ---

    println!("running api requests");
    run_bruno_requests().await?;

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
