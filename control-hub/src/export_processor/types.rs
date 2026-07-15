use std::collections::HashMap;

use chrono::{DateTime, Utc};
use clickhouse::{Client, Row, insert::Insert};
use control_core::{ConfigMutationOrigin, ConfigMutationResult, LogLevel, ScalarValue};
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
            logs: client.insert("logs")?,
            events: client.insert("events")?,
            machine_activity: client.insert("machine_activity")?,
            config_mutations: client.insert("config_mutations")?,
            state_mutations: client.insert("state_mutations")?,
            machine_measurements: client.insert("machine_measurements")?,
        })
    }

    pub async fn end(self) -> clickhouse::error::Result<()> {
        tokio::try_join!(
            self.logs.end(),
            self.events.end(),
            self.machine_activity.end(),
            self.config_mutations.end(),
            self.state_mutations.end(),
            self.machine_measurements.end(),
        )?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Row)]
pub struct LogRecordRow {
    #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub origin: u64,
    pub message: String,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Row)]
pub struct EventRecordRow {
    #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
    pub timestamp: DateTime<Utc>,
    pub origin: u64,
    pub name: String,
    pub value: serde_json::Value,
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

    pub value_type: ScalarValueType,
    pub value_enum: Option<String>,
    pub value_string: Option<String>,
    pub value_int: Option<i64>,
    pub value_float: Option<f64>,
    pub value_bool: Option<bool>,

    pub origin: ConfigMutationOrigin,
    pub result: ConfigMutationResult,
}

#[derive(Debug, Serialize, Deserialize, Row)]
pub struct StateMutationRecordRow {
    #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
    pub timestamp: DateTime<Utc>,
    pub identity: u64,
    pub name: String,

    pub value_type: ScalarValueType,
    pub value_enum: Option<String>,
    pub value_string: Option<String>,
    pub value_bool: Option<bool>,
    pub value_int: Option<i64>,
    pub value_float: Option<f64>,
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
    pub value_enum: Option<String>,
    pub value_string: Option<String>,
    pub value_int: Option<i64>,
    pub value_float: Option<f64>,
    pub value_bool: Option<bool>,
}

impl From<&ScalarValue> for ScalarValueColumns {
    fn from(value: &ScalarValue) -> Self {
        let mut columns = ScalarValueColumns {
            value_type: value.into(),
            value_string: None,
            value_int: None,
            value_float: None,
            value_bool: None,
            value_enum: None,
        };

        match value {
            ScalarValue::Enum(v) => columns.value_enum = v.clone(),
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