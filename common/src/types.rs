use std::borrow::Cow;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(i8)]
pub enum ScalarValueKind {
    Enum = 1,
    String = 2,
    Boolean = 3,
    Integer = 4,
    Float = 5,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScalarValue {
    Enum(Option<Cow<'static, str>>),
    String(Option<Cow<'static, str>>),
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

    pub fn r#enum(self) -> Option<Cow<'static, str>> {
        match self {
            ScalarValue::Enum(value) => value,
            other => panic!("expected Enum, got {:?}", other),
        }
    }

    pub fn string(self) -> Option<Cow<'static, str>> {
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OperationOrigin {
    Request { request_id: u64 },
    Machine,
}

impl From<OperationOrigin> for u64 {
    fn from(value: OperationOrigin) -> Self {
        match value {
            OperationOrigin::Request { request_id } => request_id,
            OperationOrigin::Machine => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(i8)]
pub enum OperationResult {
    Success = 0,
    Failure = 1,
}

impl From<OperationResult> for i8 {
    fn from(v: OperationResult) -> Self {
        v as i8
    }
}

impl TryFrom<i8> for OperationResult {
    type Error = String;

    fn try_from(v: i8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Self::Success),
            1 => Ok(Self::Failure),
            _ => Err(format!("invalid ConfigMutationOrigin: {v}")),
        }
    }
}

// --- uom quanties ---
