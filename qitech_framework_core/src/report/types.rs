use core::fmt;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

use crate::ScalarValue;
use crate::ident::MachineIdentificationUnique;
use crate::report::ResourceKind;

// --- record ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord<T> {
    pub timestamp: DateTime<Utc>,
    pub machine: MachineIdentificationUnique,
    pub path: String,
    pub event: T,
}

// --- resource error ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Error)]
pub enum ResourceAccessError {
    #[error("machine not found")]
    MachineNotFound,

    #[error("resource not found: {kind} at '{path}'")]
    ResourceNotFound { kind: ResourceKind, path: String },

    #[error("resource type mismatch: expected {expected}, received {actual}")]
    TypeMismatch { expected: String, actual: String },
}

// --- write capability ---
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum OperationCapability {
    #[default]
    Allowed,
    Forbidden {
        reason: String,
    },
}

impl OperationCapability {
    pub fn allowed() -> Self {
        Self::Allowed
    }

    pub fn forbidden(reason: impl ToString) -> Self {
        Self::Forbidden {
            reason: reason.to_string(),
        }
    }

    pub const fn is_allowed(&self) -> bool {
        matches!(self, OperationCapability::Allowed)
    }

    pub const fn is_forbidden(&self) -> bool {
        matches!(self, OperationCapability::Forbidden { .. })
    }
}

impl fmt::Display for OperationCapability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Allowed => write!(f, "Allowed"),
            Self::Forbidden { reason } => write!(f, "Forbidden: {reason}"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OperationOrigin {
    Request { request_id: u64 },
    Machine,
}

impl std::fmt::Display for OperationOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationOrigin::Request { request_id } => {
                write!(f, "Request ({request_id})")
            }
            OperationOrigin::Machine => {
                write!(f, "Machine")
            }
        }
    }
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
        min: ScalarValue,
        max: ScalarValue,
    },
    String {
        min_length: Option<usize>,
        max_length: usize,
        pattern: Option<String>,
    },
    Enum {
        allowed: Vec<String>,
    },
}

impl fmt::Display for Constraints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "None"),

            Self::Numeric { min, max } => write!(f, "[{min}, {max}]",),

            Self::String {
                min_length,
                max_length,
                pattern,
            } => {
                if let Some(min) = min_length {
                    write!(f, " min_length={min}")?;
                }

                write!(f, " max_length={max_length}")?;

                if let Some(pattern) = pattern {
                    write!(f, " pattern={pattern:?}")?;
                }

                Ok(())
            }

            Self::Enum { allowed } => {
                write!(f, "[")?;

                for (i, value) in allowed.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{value}")?;
                }

                write!(f, "]")?;

                Ok(())
            }
        }
    }
}

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintViolationError {
    #[error("value type does not match the expected type")]
    TypeMismatch,

    #[error("value {value} is below the minimum allowed value {min}")]
    BelowMin {
        value: ScalarValue,
        min: ScalarValue,
    },

    #[error("value {value} is above the maximum allowed value {max}")]
    AboveMax {
        value: ScalarValue,
        max: ScalarValue,
    },

    #[error("invalid constraint range: minimum {min} is greater than maximum {max}")]
    IllegalRange { min: ScalarValue, max: ScalarValue },

    #[error("no allowed values are configured")]
    NoAllowedVariants,

    #[error("value {value} cannot be null")]
    CannotBeNull { value: ScalarValue },

    #[error("string length {length} is below the minimum allowed length {min}")]
    StringTooShort { length: usize, min: usize },

    #[error("string length {length} exceeds the maximum allowed length {max}")]
    StringTooLong { length: usize, max: usize },

    #[error("string does not match the required pattern: {pattern}")]
    IllegalPattern { pattern: String, error: String },

    #[error("string does not match the required pattern: {pattern}")]
    PatternMismatch { pattern: String },

    #[error("value {value:?} is not one of the allowed variants")]
    ForbiddenVariant { value: ScalarValue },
}
