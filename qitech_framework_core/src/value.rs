use core::fmt;

use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(i8)]
pub enum ScalarValueKind {
    Null = 0,
    String = 1,
    Boolean = 2,
    Integer = 3,
    IntegerUnsigned = 4,
    Float = 5,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScalarValue {
    Null,
    String(String),
    Boolean(bool),
    Integer(i64),
    IntegerUnsigned(u64),
    Float(f64),
}

impl ScalarValue {
    pub fn kind(&self) -> ScalarValueKind {
        use ScalarValue::*;

        match self {
            Null => ScalarValueKind::Null,
            String(_) => ScalarValueKind::String,
            Boolean(_) => ScalarValueKind::Boolean,
            Integer(_) => ScalarValueKind::Integer,
            IntegerUnsigned(_) => ScalarValueKind::Integer,
            Float(_) => ScalarValueKind::Float,
        }
    }

    pub fn expect_string(self) -> Option<String> {
        match self {
            ScalarValue::Null => None,
            ScalarValue::String(value) => Some(value),
            other => panic!("expected String, got {:?}", other),
        }
    }

    pub fn expect_integer(self) -> Option<i64> {
        match self {
            ScalarValue::Null => None,
            ScalarValue::Integer(value) => Some(value),
            other => panic!("expected Integer, got {:?}", other),
        }
    }

    pub fn expect_float(self) -> Option<f64> {
        match self {
            ScalarValue::Null => None,
            ScalarValue::Float(value) => Some(value),
            other => panic!("expected Float, got {:?}", other),
        }
    }

    pub fn expect_boolean(self) -> Option<bool> {
        match self {
            ScalarValue::Null => None,
            ScalarValue::Boolean(value) => Some(value),
            other => panic!("expected Boolean, got {:?}", other),
        }
    }
}

impl fmt::Display for ScalarValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScalarValue::Null => write!(f, "null"),
            ScalarValue::String(v) => write!(f, "{v}"),
            ScalarValue::Boolean(v) => write!(f, "{v}"),
            ScalarValue::Integer(v) => write!(f, "{v}"),
            ScalarValue::IntegerUnsigned(v) => write!(f, "{v}"),
            ScalarValue::Float(v) => write!(f, "{v:.2}"),
        }
    }
}

/// Error when writing fails due to Scalar Value being the wrong representation
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
#[error("scalar value has an incompatible type")]
pub struct ScalarValueTypeMismatchError;
