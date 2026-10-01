use core::fmt;

use serde::Deserialize;
use serde::Serialize;

// --- capability ---
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

// --- origin ---
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OperationOrigin {
    Request { request_id: u64 },
    Machine,
}

impl fmt::Display for OperationOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
