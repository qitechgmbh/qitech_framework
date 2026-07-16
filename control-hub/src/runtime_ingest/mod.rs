use anyhow::bail;
use clickhouse::inserter::Inserter;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use chrono::{DateTime, Utc};
use control_core::{
    ConfigMutationOrigin, ConfigMutationRecord, LogOrigin, LogRecord, 
    MachineEvent, Measurements, RuntimeEvent, RuntimeEventKind, RuntimeExport, 
    StateMutationRecord,
};

mod types;
use types::{Inserters, ScalarValueColumns};

use tokio::{sync::broadcast, time::timeout};
use crate::{SharedState, tables};
use crate::machine_registry::{self, MachineRegistry};
// use crate::runtime_ingest::types::;

pub async fn run(state: SharedState) -> anyhow::Result<()> {
    let export_interval = state.config.export_interval;

    // create local copy we can modify
    let mut machines = (*state.machines.load_full()).clone();

    // subscribe to incoming exports
    let mut rx = state.data_tx.subscribe();

    // create inserters for the database
    let mut inserters = Inserters::new(&state.client, export_interval).await?;

    // track the last export
    let mut last_export_ts = Instant::now();

    loop {
        let now = Instant::now();

        if now.duration_since(last_export_ts) >= export_interval {
            // reached export interval, commit all changes

            let start = Instant::now();

            inserters.commit_all().await?;

            last_export_ts = now;
            println!("[runtime_ingest] Committed data, took {:?}", start.elapsed());
        }

        // receive the next export batch
        if let Ok(result) = timeout(Duration::from_millis(100), rx.recv()).await {
            use broadcast::error::RecvError;

            match result {
                Ok(data) => {
                    process_export(
                        &state,
                        &mut machines,
                        &mut inserters,
                        data,
                    ).await?;
                }
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

async fn process_export(
    state: &SharedState,
    machines: &mut MachineRegistry,
    inserters: &mut Inserters,
    export: Arc<RuntimeExport>,
) -> anyhow::Result<()> {
    process_logs(&mut inserters.logs, &export.logs).await?;

    process_runtime_events(
        state, 
        machines, 
        &mut inserters.events, 
        &export.runtime_events
    ).await?;

    process_machine_events(
        &mut inserters.events, 
        &export.machine_events,
    ).await?;

    process_machine_config_mutations(
        machines, 
        &mut inserters.machine_config_mutations, 
        &export.config_mutations
    ).await?;

    process_machine_state_mutations(
        machines, 
        &mut inserters.machine_state_mutations,
        &export.state_mutations
    ).await?;

    process_machine_measurements(
        export.created_at,
        machines,
        &mut inserters.machine_measurements,
        &export.machine_measurements,
    ).await?;

    // finally replace the outdated shared registry with the new one
    state.machines.swap(Arc::new(machines.clone()));
    Ok(())
}

async fn process_logs(
    inserter: &mut Inserter<tables::logs::Row>, 
    records: &Vec<LogRecord>
) -> anyhow::Result<()> {
    for record in records {
        let origin = match record.origin {
            LogOrigin::MainLoop => 0,
            LogOrigin::Machine(ident) => ident.to_u64(),
        };

        let attributes = record.attributes
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        inserter.write(&tables::logs::Row {
            timestamp: record.timestamp,
            origin,
            level: record.level as i8,
            message: record.message.clone(),
            attributes,
        }).await?;
    }

    Ok(())
}

async fn process_runtime_events(
    state: &SharedState,
    machines: &mut MachineRegistry,
    inserter: &mut Inserter<tables::events::Row>,
    events: &Vec<RuntimeEvent>,
) -> anyhow::Result<()> {
    for event in events {
        let value = match event.kind {
            RuntimeEventKind::MachineConnected(ident) => {
                machine_registry::mark_connected(machines, state, ident)?;
                serde_json::to_string(&ident)?
            }

            RuntimeEventKind::MachineDisconnected(ident) => {
                machine_registry::mark_disconnected(machines, ident);
                serde_json::to_string(&ident)?
            }
        };

        inserter.write(&tables::events::Row {
            timestamp: event.timestamp,
            origin: 0,
            name: event.kind.to_string(),
            value,
        }).await?;
    }

    Ok(())
}

async fn process_machine_events(
    inserter: &mut Inserter<tables::events::Row>,
    events: &Vec<MachineEvent>,
) -> anyhow::Result<()> {
    for MachineEvent {
        timestamp,
        ident,
        name,
        data,
    } in events
    {
        inserter.write(&tables::events::Row {
            timestamp: *timestamp,
            origin: ident.to_u64(),
            name: name.to_string(),
            value: data.clone(),
        }).await?;
    }

    Ok(())
}

async fn process_machine_config_mutations(
    machines: &mut MachineRegistry,
    inserter: &mut Inserter<tables::machine_config_mutations::Row>,
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

        inserter.write(&tables::machine_config_mutations::Row {
            timestamp: record.timestamp,
            identity: record.ident.to_u64(),
            name: record.name.to_string(),
            value_type: value_type as i8,
            value_enum,
            value_string,
            value_int,
            value_float,
            value_bool,
            origin: match record.origin {
                ConfigMutationOrigin::User { request_id } => request_id,
                ConfigMutationOrigin::Machine => 0,
            },
            result: record.result as i8,
        }).await?;
    }

    Ok(())
}

async fn process_machine_state_mutations(
    machines: &mut MachineRegistry,
    inserter: &mut Inserter<tables::machine_state_mutations::Row>,
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

        inserter.write(&tables::machine_state_mutations::Row {
            timestamp: record.timestamp,
            identity: record.ident.to_u64(),
            name: record.name.to_string(),
            value_type: value_type as i8,
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
    inserter: &mut Inserter<tables::machine_measurements::Row>,
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

        inserter.write(&tables::machine_measurements::Row {
            timestamp,
            identity: snapshot.ident.to_u64(),
            name: snapshot.name.to_string(),
            value: *prop,
        }).await?;
    }

    Ok(())
}
