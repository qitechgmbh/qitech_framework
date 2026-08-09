use core::fmt;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

use crate::NumericValue;
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

    #[error("resource not found")]
    ResourceNotFound { kind: ResourceKind, path: String },

    #[error("resource type mismatch: expected {actual}, received {received}")]
    TypeMismatch { actual: String, received: String },
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
        allowed: Vec<ScalarValue>,
        nullable: bool,
    },
}

impl fmt::Display for Constraints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "None"),

            Self::Numeric { min, max, nullable } => write!(
                f,
                "[{min}, {max}]{}",
                if *nullable { " nullable" } else { "" }
            ),

            Self::String {
                min_length,
                max_length,
                pattern,
                nullable,
            } => {
                if let Some(min) = min_length {
                    write!(f, " min_length={min}")?;
                }

                if let Some(max) = max_length {
                    write!(f, " max_length={max}")?;
                }

                if let Some(pattern) = pattern {
                    write!(f, " pattern={pattern:?}")?;
                }

                if *nullable {
                    write!(f, " nullable")?;
                }

                Ok(())
            }

            Self::Enum { allowed, nullable } => {
                write!(f, "[")?;

                for (i, value) in allowed.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }

                    write!(f, "{value}")?;
                }

                write!(f, "]")?;

                if *nullable {
                    write!(f, " nullable")?;
                }

                Ok(())
            }
        }
    }
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

    #[error("...")]
    NoAllowedVariants,

    #[error("value {value} cannot be null")]
    CannotBeNull { value: ScalarValue },

    #[error("string length {length} is below the minimum {min}")]
    StringTooShort { length: usize, min: usize },

    #[error("string length {length} is above the maximum {max}")]
    StringTooLong { length: usize, max: usize },

    #[error("string does not match required pattern: {pattern}")]
    PatternMismatch { pattern: String },

    #[error("value {value:?} is not one of the allowed enum variants")]
    ForbiddenVariant { value: ScalarValue },
}
