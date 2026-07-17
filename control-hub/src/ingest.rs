use std::{sync::Arc, time::{Duration, Instant}};
use anyhow::bail;
use chrono::{DateTime, Utc};
use clickhouse::inserter::Inserter;
use control_core::{LogRecord, MachineConfigMutation, MachineEvent, MachineMeasurementVec, MachineStateMutation, Origin, RuntimeEvent, RuntimeEventKind, RuntimeReport, ScalarValue, ScalarValueKind};
use tokio::{sync::broadcast, time::timeout};
use crate::{SharedState, machine_registry::MachineRegistry, tables};

const MAX_ROWS: u64 = 10_000;

pub struct IngestManager {
    state: SharedState,

    // local mutable copy
    machines: MachineRegistry,

    // --- inserters ---
    logs: Inserter<tables::logs::Row>,
    events: Inserter<tables::events::Row>,
    machine_activity: Inserter<tables::machine_activity::Row>,
    machine_config_mutations: Inserter<tables::machine_config_mutations::Row>,
    machine_state_mutations: Inserter<tables::machine_state_mutations::Row>,
    machine_measurements: Inserter<tables::machine_measurements::Row>,
}

impl IngestManager {
    pub fn init(state: &SharedState) -> Self {
        macro_rules! define_inserter {
            ($mod:tt) => {
                state.client
                    .inserter::<tables::$mod::Row>(tables::$mod::TABLE_NAME)
                    .with_period(Some(state.config.commit_interval))
                    .with_max_rows(MAX_ROWS)
            };
        }

        let machines = (*state.machines.load_full()).clone();

        let logs = define_inserter!(logs);
        let events = define_inserter!(events);
        let machine_activity = define_inserter!(machine_activity);
        let machine_config_mutations = define_inserter!(machine_config_mutations);
        let machine_state_mutations = define_inserter!(machine_state_mutations);
        let machine_measurements = define_inserter!(machine_measurements);

        Self { 
            state: state.clone(),
            machines,
            logs,
            events,
            machine_activity,
            machine_config_mutations,
            machine_state_mutations,
            machine_measurements,
        }
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        let export_interval = self.state.config.commit_interval;
        let mut report_rx = self.state.report_tx.subscribe();

        // track the last export
        let mut last_export_ts = Instant::now();

        loop {
            let now = Instant::now();

            if now.duration_since(last_export_ts) >= export_interval {
                let start = Instant::now();
                self.commit_all().await?;
                println!("[IngestManager] Committed data, took {:?}", start.elapsed());

                last_export_ts = now;
            }

            // receive the next export batch
            if let Ok(result) = timeout(Duration::from_millis(100), report_rx.recv()).await {
                use broadcast::error::RecvError;

                match result {
                    Ok(report) => self.process_report(report).await?,
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

    async fn commit_all(&mut self) -> anyhow::Result<()> {
        macro_rules! timed {
            ($name:expr, $future:expr) => {{
                async {
                    let start = Instant::now();
                    let result = $future.await;

                    println!("elapsed: {} for {:?}", $name, start.elapsed());
                    result
                }
            }};
        }

        tokio::try_join!(
            timed!("logs.commit", self.logs.commit()),
            timed!("events.commit", self.events.commit()),
            timed!("machine_activity.commit", self.machine_activity.commit()),
            timed!("machine_config_mutations.commit", self.machine_config_mutations.commit()),
            timed!("machine_state_mutations.commit", self.machine_state_mutations.commit()),
            timed!("machine_measurements.commit", self.machine_measurements.commit()),
        )?;

        Ok(())
    }

    async fn process_report(
        &mut self,
        report: Arc<RuntimeReport>,
    ) -> anyhow::Result<()> {
        self.process_logs(&report.logs).await?;
        self.process_runtime_events(&report.runtime.events).await?;
        self.process_machine_events(&report.machines.events).await?;
        self.process_machine_config_mutations(&report.machines.config_mutations).await?;
        self.process_machine_state_mutations(&report.machines.state_mutations).await?;
        self.process_machine_measurements(report.created_at, &report.machines.measurements).await?;

        // finally replace the outdated shared registry with the new one
        self.state.machines.swap(Arc::new(self.machines.clone()));
        Ok(())
    }

    async fn process_logs(&mut self, records: &Vec<LogRecord>) -> anyhow::Result<()> {
        for record in records {
            let origin = record.origin.to_u64();

            let attributes = record.attributes
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            self.logs.write(&tables::logs::Row {
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
        &mut self,
        events: &Vec<RuntimeEvent>,
    ) -> anyhow::Result<()> {
        for event in events {
            use RuntimeEventKind::*;
            match &event.kind {
                MachineConnected { ident } => {
                    let schemas = self.state.schemas.load();
                    self.machines.mark_connected(&schemas, *ident)?;
                },
                MachineDisconnected { ident } => {
                    self.machines.mark_disconnected(*ident);
                },
                _ => {}
            }

            self.events.write(&tables::events::Row {
                timestamp: event.timestamp,
                origin: 0,
                name: "".into(), // TODO: USE KIND AS TAG
                value: "".to_string(), // TODO: EXPORT
            }).await?;
        }

        Ok(())
    }

    async fn process_machine_events(
        &mut self,
        events: &Vec<MachineEvent>,
    ) -> anyhow::Result<()> {
        for MachineEvent {
            timestamp,
            ident,
            name,
            data,
        } in events
        {
            self.events.write(&tables::events::Row {
                timestamp: *timestamp,
                origin: ident.to_u64(),
                name: name.to_string(),
                value: data.to_string(),
            }).await?;
        }

        Ok(())
    }

    async fn process_machine_config_mutations(
        &mut self,
        mutations: &Vec<MachineConfigMutation>,
    ) -> anyhow::Result<()> {
        for mutation in mutations {
            let Some(machine) = self.machines.get_mut(mutation.ident) else {
                bail!(
                    "Exported config mutation for non existing machine {}",
                    mutation.ident
                );
            };

            let Some(prop) = machine.properties.config.get_mut(mutation.name.as_ref()) else {
                bail!(
                    "Exported config mutation for non existing property {} of machine {}",
                    mutation.name.as_ref(),
                    mutation.ident,
                );
            };

            // update cached value
            *prop = mutation.value.clone();

            let ScalarValueColumns {
                value_type,
                value_enum,
                value_string,
                value_int,
                value_float,
                value_bool,
            } = ScalarValueColumns::from(&mutation.value);

            self.machine_config_mutations.write(&tables::machine_config_mutations::Row {
                timestamp: mutation.timestamp,
                identity: mutation.ident.to_u64(),
                name: mutation.name.to_string(),
                value_type: value_type as i8,
                value_enum,
                value_string,
                value_int,
                value_float,
                value_bool,
                origin: match mutation.origin {
                    Origin::Request { request_id } => request_id,
                    Origin::Machine => 0,
                },
                result: mutation.result as i8,
            }).await?;
        }

        Ok(())
    }

    async fn process_machine_state_mutations(
        &mut self,
        records: &Vec<MachineStateMutation>,
    ) -> anyhow::Result<()> {
        for record in records {
            let Some(machine) = self.machines.get_mut(record.ident) else {
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

            self.machine_state_mutations.write(&tables::machine_state_mutations::Row {
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
        &mut self,
        timestamp: DateTime<Utc>,
        measurements: &MachineMeasurementVec,
    ) -> anyhow::Result<()> {
        for snapshot in measurements {
            let Some(machine) = self.machines.get_mut(*snapshot.ident) else {
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

            self.machine_measurements.write(&tables::machine_measurements::Row {
                timestamp,
                identity: snapshot.ident.to_u64(),
                name: snapshot.name.to_string(),
                value: *prop,
            }).await?;
        }

        Ok(())
    }
}

// --- misc ---

pub struct ScalarValueColumns {
    pub value_type: ScalarValueKind,
    pub value_enum: String,
    pub value_string: Option<String>,
    pub value_bool: Option<bool>,
    pub value_int: Option<i64>,
    pub value_float: Option<f64>,
}

impl From<&ScalarValue> for ScalarValueColumns {
    fn from(value: &ScalarValue) -> Self {
        let mut columns = ScalarValueColumns {
            value_type: value.kind(),
            value_enum: "".into(),
            value_string: None,
            value_int: None,
            value_float: None,
            value_bool: None,
        };

        match value {
            ScalarValue::Enum { value } => columns.value_enum = value.clone(),
            ScalarValue::String { value } => columns.value_string = value.clone(),
            ScalarValue::Boolean { value } => columns.value_bool = *value,
            ScalarValue::Integer { value } => columns.value_int = *value,
            ScalarValue::Float { value } => columns.value_float = *value,
        };

        columns
    }
}

impl From<ScalarValue> for ScalarValueColumns {
    fn from(value: ScalarValue) -> Self {
        (&value).into()
    }
}
