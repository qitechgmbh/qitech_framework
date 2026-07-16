use std::time::{Duration, Instant};

use clickhouse::{Client, inserter::Inserter};
use control_core::ScalarValue;
use serde::{Deserialize, Serialize};

use crate::tables;

const MAX_ROWS: u64 = 10_000;

pub struct Inserters {
    pub logs: Inserter<tables::logs::Row>,
    pub events: Inserter<tables::events::Row>,
    pub machine_activity: Inserter<tables::machine_activity::Row>,
    pub machine_config_mutations: Inserter<tables::machine_config_mutations::Row>,
    pub machine_state_mutations: Inserter<tables::machine_state_mutations::Row>,
    pub machine_measurements: Inserter<tables::machine_measurements::Row>,
}

impl Inserters {
    pub async fn new(client: &Client, export_interval: Duration) -> anyhow::Result<Self> {
        macro_rules! define_inserter {
            ($mod:tt) => {
                client
                    .inserter::<tables::$mod::Row>(tables::$mod::TABLE_NAME)
                    .with_period(Some(export_interval))
                    .with_max_rows(MAX_ROWS)
            };
        }

        Ok(Self {
            logs: define_inserter!(logs),
            events: define_inserter!(events),
            machine_activity: define_inserter!(machine_activity),
            machine_config_mutations: define_inserter!(machine_config_mutations),
            machine_state_mutations: define_inserter!(machine_state_mutations),
            machine_measurements: define_inserter!(machine_measurements),
        })
    }

    pub async fn commit_all(&mut self) -> anyhow::Result<()> {
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
}

// --- misc ---

#[derive(Debug, Serialize, Deserialize)]
#[repr(i8)]
pub enum ScalarValueType {
    Enum,
    String,
    Integer,
    IntegerUnsigned,
    Float,
    Boolean,
}

impl From<&ScalarValue> for ScalarValueType {
    fn from(value: &ScalarValue) -> Self {
        match value {
            ScalarValue::Enum(_) => ScalarValueType::Enum,
            ScalarValue::String(_) => ScalarValueType::String,
            ScalarValue::Integer(_) => ScalarValueType::Integer,
            ScalarValue::Float(_) => ScalarValueType::Float,
            ScalarValue::Boolean(_) => ScalarValueType::Boolean,
        }
    }
}

impl From<ScalarValue> for ScalarValueType {
    fn from(value: ScalarValue) -> Self {
        (&value).into()
    }
}

pub struct ScalarValueColumns {
    pub value_type: ScalarValueType,
    pub value_enum: String,
    pub value_string: Option<String>,
    pub value_int: Option<i64>,
    pub value_float: Option<f64>,
    pub value_bool: Option<bool>,
}

impl From<&ScalarValue> for ScalarValueColumns {
    fn from(value: &ScalarValue) -> Self {
        let mut columns = ScalarValueColumns {
            value_type: value.into(),
            value_enum: "".into(),
            value_string: None,
            value_int: None,
            value_float: None,
            value_bool: None,
        };

        match value {
            ScalarValue::Enum(v) => {
                columns.value_enum = v.clone().expect("MUST");
            },
            ScalarValue::String(v) => columns.value_string = v.clone(),
            ScalarValue::Integer(v) => columns.value_int = *v,
            ScalarValue::Float(v) => columns.value_float = *v,
            ScalarValue::Boolean(v) => columns.value_bool = *v,
        }

        columns
    }
}

impl From<ScalarValue> for ScalarValueColumns {
    fn from(value: ScalarValue) -> Self {
        (&value).into()
    }
}