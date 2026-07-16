use serde::{Deserialize, Serialize};

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
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ScalarValue {
    Enum { value: String },
    String { value: Option<String> },
    Boolean { value: Option<bool> },
    Integer { value: Option<i64> },
    Float { value: Option<f64> },
}

impl ScalarValue {
    pub fn kind(&self) -> ScalarValueKind {
        use ScalarValue::*;
        match self {
            Enum { .. } => ScalarValueKind::String,
            String { .. } => ScalarValueKind::String,
            Boolean { .. } => ScalarValueKind::Boolean,
            Integer { .. } => ScalarValueKind::Integer,
            Float { .. } => ScalarValueKind::Float,
        }
    }

    pub fn r#enum(self) -> Option<String> {
        match self {
            ScalarValue::String { value } => value,
            other => panic!("expected Enum, got {:?}", other),
        }
    }

    pub fn string(self) -> Option<String> {
        match self {
            ScalarValue::String { value } => value,
            other => panic!("expected String, got {:?}", other),
        }
    }

    pub fn integer(self) -> Option<i64> {
        match self {
            ScalarValue::Integer { value } => value,
            other => panic!("expected Integer, got {:?}", other),
        }
    }

    pub fn float(self) -> Option<f64> {
        match self {
            ScalarValue::Float { value } => value,
            other => panic!("expected Float, got {:?}", other),
        }
    }

    pub fn boolean(self) -> Option<bool> {
        match self {
            ScalarValue::Boolean { value } => value,
            other => panic!("expected Boolean, got {:?}", other),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Origin {
    Request { request_id: u64 },
    Machine,
}

impl From<Origin> for u64 {
    fn from(value: Origin) -> Self {
        match value {
            Origin::Request { request_id } => request_id,
            Origin::Machine => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(i8)]
pub enum OperationResult {
    Success = 0,
    OutOfBounds = 1,
    InvalidInput = 2,
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
            1 => Ok(Self::OutOfBounds),
            2 => Ok(Self::InvalidInput),
            _ => Err(format!("invalid ConfigMutationOrigin: {v}")),
        }
    }
}