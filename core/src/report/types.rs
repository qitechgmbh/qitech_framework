use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

use crate::NumericValue;
use crate::ScalarValue;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum WriteCapability {
    #[default]
    Allowed,
    Forbidden {
        reason: String,
    },
}

impl WriteCapability {
    pub const fn is_allowed(&self) -> bool {
        matches!(self, WriteCapability::Allowed)
    }

    pub const fn forbidden(&self) -> bool {
        matches!(self, WriteCapability::Forbidden { .. })
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

// --- constraints ---
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum Constraints {
    #[default]
    None,
    Numeric {
        min: NumericValue,
        max: NumericValue,
        nullable: bool,
    },
    String {
        min_length: Option<usize>,
        max_length: Option<usize>,
        pattern: Option<String>,
        nullable: bool,
    },
    Enum {
        allowed: Vec<String>,
        nullable: bool,
    },
}

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintViolationError {
    #[error("types didn't match")]
    TypeMismatch,

    #[error("value {value} is below the minimum {min}")]
    BelowMin {
        value: NumericValue,
        min: NumericValue,
    },

    #[error("value {value} is above the maximum {max}")]
    AboveMax {
        value: NumericValue,
        max: NumericValue,
    },

    #[error("value {value} cannot be null")]
    CannotBeNull { value: ScalarValue },

    #[error("string length {length} is below the minimum {min}")]
    StringTooShort { length: usize, min: usize },

    #[error("string length {length} is above the maximum {max}")]
    StringTooLong { length: usize, max: usize },

    #[error("string does not match required pattern: {pattern}")]
    PatternMismatch { pattern: String },

    #[error("value {value:?} is not one of the allowed enum variants")]
    ForbiddenVariant { value: String },
}
