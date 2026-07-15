use anyhow::bail;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use chrono::{DateTime, Utc};
use control_core::{
    ConfigMutationRecord, LogOrigin, LogRecord, MachineEvent, Measurements, RuntimeEvent,
    RuntimeEventKind, RuntimeExport, StateMutationRecord,
};

mod types;
use tokio::{sync::broadcast, time::timeout};
use types::Inserts;

use crate::{
    SharedState,
    runtime_ingest::types::{
        ConfigMutationRecordRow, EventRecordRow, LogRecordRow, MeasurementSampleRow,
        ScalarValueColumns, StateMutationRecordRow,
    },
    machine_registry::{self, MachineRegistry},
};

pub async fn run(state: SharedState) -> anyhow::Result<()> {
    let export_interval = state.config.export_interval;

    // create local copies we can modify
    let mut machines = (*state.machines.load_full()).clone();

    let mut rx = state.data_tx.subscribe();
    let mut last_export_ts = Instant::now();

    loop {
        let mut inserts = Inserts::new(&state.client).await?;

        loop {
            let now = Instant::now();

            if now.duration_since(last_export_ts) >= export_interval {
                println!("Exporting");
                inserts.end().await?;
                last_export_ts = now;
                break;
            }

            // receive the next export batch
            if let Ok(result) = timeout(Duration::from_millis(100), rx.recv()).await {
                use broadcast::error::RecvError;

                match result {
                    Ok(data) => process_export(&state, &mut machines, &mut inserts, data).await?,
                    Err(e) => match e {
                        RecvError::Closed => return Ok(()),
                        RecvError::Lagged(count) => {
                            eprintln!("Lagged behind {count} messages!");
                            continue;
                        }
                    },
                }
            }
        }
    }
}

async fn process_export(
    state: &SharedState,
    machines: &mut MachineRegistry,
    inserts: &mut Inserts,
    export: Arc<RuntimeExport>,
) -> anyhow::Result<()> {
    process_logs(inserts, &export.logs).await?;
    process_runtime_events(state, machines, inserts, &export.runtime_events).await?;
    process_machine_events(inserts, &export.machine_events).await?;
    process_machine_config_mutations(machines, inserts, &export.config_mutations).await?;
    process_machine_state_mutations(machines, inserts, &export.state_mutations).await?;
    process_machine_measurements(
        export.created_at,
        machines,
        inserts,
        &export.machine_measurements,
    ).await?;

    // TODO: check which machines are still online and create a last online entry

    // finally update the shared registry with the new one
    state.machines.swap(Arc::new(machines.clone()));
    Ok(())
}

async fn process_logs(inserts: &mut Inserts, records: &Vec<LogRecord>) -> anyhow::Result<()> {
    for record in records {
        let origin = match record.origin {
            LogOrigin::MainLoop => 0,
            LogOrigin::Machine(ident) => ident.to_u64(),
        };

        let attributes = record.attributes
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        inserts.logs.write(&LogRecordRow {
            timestamp: record.timestamp,
            origin,
            level: record.level,
            message: record.message.clone(),
            attributes,
        }).await?;
    }

    Ok(())
}

async fn process_runtime_events(
    state: &SharedState,
    machines: &mut MachineRegistry,
    inserts: &mut Inserts,
    events: &Vec<RuntimeEvent>,
) -> anyhow::Result<()> {
    for event in events {
        let value = match event.kind {
            RuntimeEventKind::MachineConnected(ident) => {
                machine_registry::insert(machines, state, ident)?;
                serde_json::to_string(&ident)?
            }

            RuntimeEventKind::MachineDisconnected(ident) => {
                machine_registry::mark_disconnected(machines, ident);
                serde_json::to_string(&ident)?
            }
        };

        inserts.events.write(&EventRecordRow {
            timestamp: event.timestamp,
            origin: 0,
            name: event.kind.to_string(),
            value,
        }).await?;
    }

    Ok(())
}

async fn process_machine_events(inserts: &mut Inserts, events: &Vec<MachineEvent>) -> anyhow::Result<()> {
    for MachineEvent {
        timestamp,
        ident,
        name,
        data,
    } in events
    {
        inserts.events.write(&EventRecordRow {
            timestamp: *timestamp,
            origin: ident.to_u64(),
            name: name.to_string(),
            value: serde_json::to_string(data)?,
        }).await?;
    }

    Ok(())
}

async fn process_machine_config_mutations(
    machines: &mut MachineRegistry,
    inserts: &mut Inserts,
    records: &Vec<ConfigMutationRecord>,
) -> anyhow::Result<()> {
    for record in records {
        let Some(machine) = machines.get_mut(&record.ident) else {
            bail!(
                "Exported config mutation for non existing machine {}",
                record.ident
            );
        };

        let Some(prop) = machine.properties.config.get_mut(record.name.as_ref()) else {
            bail!(
                "Exported config mutation for non existing property {} of machine {}",
                record.name.as_ref(),
                record.ident,
            );
        };

        // update cached value
        *prop = record.value.clone();

        let ScalarValueColumns {
            value_type,
            value_enum,
            value_string,
            value_int,
            value_float,
            value_bool,
        } = ScalarValueColumns::from(&record.value);

        inserts.config_mutations.write(&ConfigMutationRecordRow {
            timestamp: record.timestamp,
            identity: record.ident.to_u64(),
            name: record.name.to_string(),
            value_type,
            value_enum,
            value_string,
            value_int,
            value_float,
            value_bool,
            origin: record.origin,
            result: record.result,
        }).await?;
    }

    Ok(())
}

async fn process_machine_state_mutations(
    machines: &mut MachineRegistry,
    inserts: &mut Inserts,
    records: &Vec<StateMutationRecord>,
) -> anyhow::Result<()> {
    for record in records {
        let Some(machine) = machines.get_mut(&record.ident) else {
            bail!(
                "Exported state mutation for non existing machine {}",
                record.ident
            );
        };

        let Some(prop) = machine.properties.state.get_mut(record.name.as_ref()) else {
            bail!(
                "Exported state mutation for non existing property {} of machine {}",
                record.name.as_ref(),
                record.ident,
            );
        };

        // update cached value
        *prop = record.value.clone();

        let ScalarValueColumns {
            value_type,
            value_enum,
            value_string,
            value_int,
            value_float,
            value_bool,
        } = ScalarValueColumns::from(&record.value);

        inserts.state_mutations.write(&StateMutationRecordRow {
            timestamp: record.timestamp,
            identity: record.ident.to_u64(),
            name: record.name.to_string(),
            value_type,
            value_enum,
            value_string,
            value_int,
            value_float,
            value_bool,
        }).await?;
    }

    Ok(())
}

async fn process_machine_measurements(
    timestamp: DateTime<Utc>,
    machines: &mut MachineRegistry,
    inserts: &mut Inserts,
    measurements: &Measurements,
) -> anyhow::Result<()> {
    for snapshot in measurements {
        let Some(machine) = machines.get_mut(snapshot.ident) else {
            bail!(
                "Exported state mutation for non existing machine {}",
                snapshot.ident
            );
        };

        let Some(prop) = machine.properties.measurements.get_mut(snapshot.name) else {
            bail!(
                "Exported state mutation for non existing property {} of machine {}",
                &snapshot.name,
                snapshot.ident,
            );
        };

        // update cached value
        *prop = if *snapshot.null {
            None
        } else {
            Some(*snapshot.value)
        };

        inserts.machine_measurements.write(&MeasurementSampleRow {
            timestamp,
            identity: snapshot.ident.to_u64(),
            name: snapshot.name.to_string(),
            value: *prop,
        }).await?;
    }

    Ok(())
}
