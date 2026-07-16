use chrono::{DateTime, Utc};
use clickhouse::{Client, Row, insert::Insert};
use control_core::ScalarValue;
use serde::{Deserialize, Serialize};

pub struct Inserts {
    pub logs: Insert<LogRecordRow>,
    pub events: Insert<EventRecordRow>,
    pub machine_activity: Insert<MachineActivityRecordRow>,
    pub config_mutations: Insert<ConfigMutationRecordRow>,
    pub state_mutations: Insert<StateMutationRecordRow>,
    pub machine_measurements: Insert<MeasurementSampleRow>,
}

impl Inserts {
    pub async fn new(client: &Client) -> anyhow::Result<Self> {
        Ok(Self {
            logs: client.insert("logs").await?,
            events: client.insert("events").await?,
            machine_activity: client.insert("machine_activity").await?,
            config_mutations: client.insert("config_mutations").await?,
            state_mutations: client.insert("state_mutations").await?,
            machine_measurements: client.insert("machine_measurements").await?,
        })
    }

    pub async fn end(self) -> anyhow::Result<()> {
        macro_rules! timed {
            ($name:expr, $future:expr) => {{
                async {
                    let start = std::time::Instant::now();
                    let result = $future.await;

                    println!("elapsed: {} for {:?}", $name, start.elapsed());
                    result
                }
            }};
        }

        tokio::try_join!(
            timed!("logs.end", self.logs.end()),
            timed!("events.end", self.events.end()),
            timed!("machine_activity.end", self.machine_activity.end()),
            timed!("config_mutations.end", self.config_mutations.end()),
            timed!("state_mutations.end", self.state_mutations.end()),
            timed!("machine_measurements.end", self.machine_measurements.end()),
        )?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Row)]
pub struct LogRecordRow {
    #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
    pub timestamp: DateTime<Utc>,

    pub level: i8,

    pub origin: u64,
    pub message: String,

    // maps are not supported by clickhouse
    pub attributes: Vec<(String, String)>,
}

#[derive(Debug, Serialize, Deserialize, Row)]
pub struct EventRecordRow {
    #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
    pub timestamp: DateTime<Utc>,
    pub origin: u64,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Row)]
pub struct MachineActivityRecordRow {
    pub identity: u64,
    #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Row)]
pub struct ConfigMutationRecordRow {
    #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
    pub timestamp: DateTime<Utc>,
    pub identity: u64,
    pub name: String,

    pub value_type: i8,
    pub value_enum: String,
    pub value_string: Option<String>,
    pub value_int: Option<i64>,
    pub value_float: Option<f64>,
    pub value_bool: Option<bool>,

    pub origin: u64,
    pub result: i8,
}

#[derive(Debug, Serialize, Deserialize, Row)]
pub struct StateMutationRecordRow {
    #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
    pub timestamp: DateTime<Utc>,
    pub identity: u64,
    pub name: String,

    pub value_type: i8,
    pub value_enum: String,
    pub value_string: Option<String>,
    pub value_int: Option<i64>,
    pub value_float: Option<f64>,
    pub value_bool: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Row)]
pub struct MeasurementSampleRow {
    #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
    pub timestamp: DateTime<Utc>,
    pub identity: u64,
    pub name: String,
    pub value: Option<f64>,
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