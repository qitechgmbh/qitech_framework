use std::time::{Duration, Instant};
use tokio::sync::watch;
use chrono::Utc;
use control_core::{MachineIdentificationUnique, RuntimeEvent, RuntimeEventKind, vendors};
use control_core::{MeasurementSnapshot, MeasurementSnapshotVec, RuntimeExport};
use control_hub::{ControlHub, Config, DatabaseConfig};
use control_hub::EmbeddedSession;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config { 
        database: DatabaseConfig {
            url: "http://localhost:8123".into(), 
            name: "control_hub".into(),
            user: "default".into(), 
            password: Some("bootstrap".into()),
        }, 
        export_interval: Duration::from_millis(2500),
        api_address: "0.0.0.0:3000".into(),
    };

    let (shutdown_tx, shutdown_rx) = watch::channel(());
    _ = shutdown_tx;

    // deploy ctrl + c hook for graceful shutdown
    // tokio::spawn(async move {
    //     if let Err(e) = tokio::signal::ctrl_c().await {
    //         eprintln!("failed to listen for ctrl-c: {e}");
    //         return;
    //     }
    // 
    //     println!("received ctrl-c, shutting down");
    //     // ignore error: if there are no receivers left, there's nothing to notify
    //     let _ = shutdown_tx.send(());
    // });

    // initialize a new hub instance
    let (app, session) = ControlHub::init_embedded(
        config, 
        vec![
            include_str!("schema.yaml").to_string(),
            include_str!("schema_2.yaml").to_string(),
        ],
        shutdown_rx,
    ).await?;

    // run app and runtime simulation
    tokio::select! {
        _ = simulate_runtime(session) => {},
        _ = app.run() => {},
    }

    Ok(())
}

async fn simulate_runtime(mut session: EmbeddedSession) {
    // give hub time to initialize
    sleep(Duration::from_millis(1000)).await;

    // Step one, emit connected machines

    let ident = MachineIdentificationUnique { 
        vendor: vendors::QITECH.id, 
        machine: 6,
        serial: 0,
    };

    let ident2 = MachineIdentificationUnique { 
        vendor: vendors::QITECH.id, 
        machine: 10,
        serial: 0,
    };

    let export = RuntimeExport {
        runtime_events: vec![
            RuntimeEvent { 
                ts: Utc::now(), 
                kind: RuntimeEventKind::MachineConnected(ident),
            },
            RuntimeEvent { 
                ts: Utc::now(), 
                kind: RuntimeEventKind::MachineConnected(ident2),
            }
        ],
        ..Default::default()
    };

    session.export(export);

    // Step two, emit diconnected machines
    sleep(Duration::from_millis(1500)).await;

    let ident2 = MachineIdentificationUnique { 
        vendor: vendors::QITECH.id, 
        machine: 10,
        serial: 0,
    };

    let export = RuntimeExport {
        runtime_events: vec![
            RuntimeEvent { 
                ts: Utc::now(), 
                kind: RuntimeEventKind::MachineDisconnected(ident2),
            }
        ],
        ..Default::default()
    };
    session.export(export);

    // step three: go into a loop

    sleep(Duration::from_millis(12_000_000)).await;
    return;

    // Step two start running them machines
    let mut then = Instant::now();

    loop {
        let now = Instant::now();

        // // read and process up to 10 request per cycle
        // for req in session.get_requests(10) {
        //     println!("[Runtime] processing request: {req:?}");
        // }

        // export once per second
        if now.duration_since(then) >= Duration::from_secs(1) {
            println!("[Runtime] exporting data");
            session.export(create_export());
            then = now;
        }

        sleep(Duration::from_millis(10)).await;
    }
}

fn create_export() -> RuntimeExport {
    let mut measurements = MeasurementSnapshotVec::new();

    // send one measurement
    measurements.push(MeasurementSnapshot {
        ident: MachineIdentificationUnique {
            vendor: 0,
            machine: 0,
            serial: 0
        },
        name: "example.measurement".to_string(),
        value: 9.0,
        null: false,
    });    

    RuntimeExport {
        created_at: Utc::now(),
        config_mutations: vec![],
        state_mutations: vec![],
        measurements,
        logs: vec![],
        runtime_events: vec![
            RuntimeEvent { 
                ts: Utc::now(), 
                kind: RuntimeEventKind::MachineConnected(MachineIdentificationUnique { vendor: 0, machine: 0, serial: 0 }),
            }
        ],
        machine_events: vec![],
    }
}
