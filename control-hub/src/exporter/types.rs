use chrono::{DateTime, Utc};
use clickhouse::Row;
use control_core::{ScalarValue, ConfigMutationOrigin, ConfigMutationResult};
use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize, Row)]
pub struct ConfigMutationRecordRow {
    #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
    pub timestamp: DateTime<Utc>,

    pub ident_vendor: u16,
    pub ident_machine: u16,
    pub ident_serial: u16,

    pub name: String,

    pub value_type: ScalarValueType,
    pub value_string: Option<String>,
    pub value_int: Option<i64>,
    pub value_float: Option<f64>,
    pub value_bool: Option<bool>,

    pub origin: ConfigMutationOrigin,
    pub result: ConfigMutationResult,
}

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