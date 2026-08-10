use core::fmt;

use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(i8)]
pub enum ScalarValueKind {
    Null = 0,
    Enum = 1,
    String = 2,
    Boolean = 3,
    Integer = 4,
    Float = 5,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScalarValue {
    Null,
    Enum(String),
    String(String),
    Boolean(bool),
    Integer(i64),
    Float(f64),
}

impl ScalarValue {
    pub fn kind(&self) -> ScalarValueKind {
        use ScalarValue::*;

        match self {
            Null => ScalarValueKind::Null,
            Enum(_) => ScalarValueKind::Enum,
            String(_) => ScalarValueKind::String,
            Boolean(_) => ScalarValueKind::Boolean,
            Integer(_) => ScalarValueKind::Integer,
            Float(_) => ScalarValueKind::Float,
        }
    }

    pub fn r#enum(self) -> Option<String> {
        match self {
            ScalarValue::Null => None,
            ScalarValue::Enum(value) => Some(value),
            other => panic!("expected Enum, got {:?}", other),
        }
    }

    pub fn string(self) -> Option<String> {
        match self {
            ScalarValue::Null => None,
            ScalarValue::String(value) => Some(value),
            other => panic!("expected String, got {:?}", other),
        }
    }

    pub fn integer(self) -> Option<i64> {
        match self {
            ScalarValue::Null => None,
            ScalarValue::Integer(value) => Some(value),
            other => panic!("expected Integer, got {:?}", other),
        }
    }

    pub fn float(self) -> Option<f64> {
        match self {
            ScalarValue::Null => None,
            ScalarValue::Float(value) => Some(value),
            other => panic!("expected Float, got {:?}", other),
        }
    }

    pub fn boolean(self) -> Option<bool> {
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
            ScalarValue::Enum(v) => write!(f, "{v}"),
            ScalarValue::String(v) => write!(f, "{v}"),
            ScalarValue::Boolean(v) => write!(f, "{v}"),
            ScalarValue::Integer(v) => write!(f, "{v}"),
            ScalarValue::Float(v) => write!(f, "{v:.2}"),
        }
    }
}

/// Error when writing fails due to Scalar Value being the wrong representation
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
#[error("scalar value has an incompatible type")]
pub struct ScalarValueTypeMismatchError;
