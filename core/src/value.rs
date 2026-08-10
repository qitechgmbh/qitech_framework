use core::fmt;

use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(i8)]
pub enum ScalarValueKind {
    Enum = 1,
    String = 2,
    Boolean = 3,
    Integer = 4,
    Float = 5,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScalarValue {
    Enum(Option<String>),
    String(Option<String>),
    Boolean(Option<bool>),
    Integer(Option<i64>),
    Float(Option<f64>),
}

impl ScalarValue {
    pub fn kind(&self) -> ScalarValueKind {
        use ScalarValue::*;

        match self {
            Enum(_) => ScalarValueKind::Enum,
            String(_) => ScalarValueKind::String,
            Boolean(_) => ScalarValueKind::Boolean,
            Integer(_) => ScalarValueKind::Integer,
            Float(_) => ScalarValueKind::Float,
        }
    }

    pub fn r#enum(self) -> Option<String> {
        match self {
            ScalarValue::Enum(value) => value,
            other => panic!("expected Enum, got {:?}", other),
        }
    }

    pub fn string(self) -> Option<String> {
        match self {
            ScalarValue::String(value) => value,
            other => panic!("expected String, got {:?}", other),
        }
    }

    pub fn integer(self) -> Option<i64> {
        match self {
            ScalarValue::Integer(value) => value,
            other => panic!("expected Integer, got {:?}", other),
        }
    }

    pub fn float(self) -> Option<f64> {
        match self {
            ScalarValue::Float(value) => value,
            other => panic!("expected Float, got {:?}", other),
        }
    }

    pub fn boolean(self) -> Option<bool> {
        match self {
            ScalarValue::Boolean(value) => value,
            other => panic!("expected Boolean, got {:?}", other),
        }
    }
}

impl fmt::Display for ScalarValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScalarValue::Enum(Some(v)) => write!(f, "{v}"),
            ScalarValue::String(Some(v)) => write!(f, "{v}"),
            ScalarValue::Boolean(Some(v)) => write!(f, "{v}"),
            ScalarValue::Integer(Some(v)) => write!(f, "{v}"),
            ScalarValue::Float(Some(v)) => write!(f, "{v:.2}"),

            ScalarValue::Enum(None)
            | ScalarValue::String(None)
            | ScalarValue::Boolean(None)
            | ScalarValue::Integer(None)
            | ScalarValue::Float(None) => write!(f, "null"),
        }
    }
}

/// Error when writing fails due to Scalar Value being the wrong representation
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
#[error("scalar value has an incompatible type")]
pub struct ScalarValueTypeMismatchError;
