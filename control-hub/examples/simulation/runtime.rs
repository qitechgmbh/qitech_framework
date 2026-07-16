use std::{borrow::Cow, time::{Duration, Instant}};

use chrono::Utc;
use control_core::{
    ConfigMutationOrigin, ConfigMutationRecord, ConfigMutationResult, LogLevel, LogOrigin, LogRecord, MachineIdentificationUnique, MeasurementSnapshot, MeasurementSnapshotVec, RuntimeEvent, RuntimeEventKind, RuntimeExport, ScalarValue, StateMutationRecord, vendors,
};
use control_hub::EmbeddedSession;
use tokio::time::sleep;

pub async fn simulate_runtime(mut session: EmbeddedSession) {
    // give hub time to initialize
    sleep(Duration::from_millis(500)).await;

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
    sleep(Duration::from_millis(250)).await;

    session.export(RuntimeExport {
        runtime_events: vec![RuntimeEvent {
            timestamp: Utc::now(),
            kind: RuntimeEventKind::MachineDisconnected(ident_m0),
        }],
        ..Default::default()
    });

    // --- Step 3: Run Loop with fuzzed data ---
    sleep(Duration::from_millis(250)).await;

    let mut then = Instant::now();

    loop {
        let now = Instant::now();

        // // read and process up to 10 request per cycle
        // for req in session.get_requests(10) {
        //     println!("[Runtime] processing request: {req:?}");
        // }

        // export once per second
        if now.duration_since(then) >= Duration::from_secs(1) {
            // println!("[Runtime] exporting data");
            session.export(create_export(ident_m1));
            then = now;
        }

        sleep(Duration::from_millis(10)).await;
    }
}

fn create_export(ident: MachineIdentificationUnique) -> RuntimeExport {
    let logs = vec![
        LogRecord {
            timestamp:Utc::now(), 
            level: LogLevel::Debug, 
            origin: LogOrigin::Machine(ident), 
            message: "Hello World".to_string(), 
            attributes: Default::default() 
        }
    ];

    let config_mutations = vec![
        ConfigMutationRecord {
            timestamp: Utc::now(),
            ident,
            name: Cow::Borrowed("temperature.target"),
            value: ScalarValue::Float(Some(22.0)),
            origin: ConfigMutationOrigin::User { request_id: 0 },
            result: ConfigMutationResult::Success,
        }
    ];

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

    RuntimeExport {
        created_at: Utc::now(),
        logs,
        config_mutations,
        state_mutations,
        machine_measurements,
        ..Default::default()
    }
}
