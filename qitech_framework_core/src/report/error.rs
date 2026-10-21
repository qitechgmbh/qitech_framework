use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

pub type ActResult = std::result::Result<(), ActError>;

#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("{message}")]
pub struct ActError {
    pub message: String,
    pub impact: ActErrorImpact,
}

impl ActError {
    pub fn ignore(message: String) -> Self {
        Self {
            message,
            impact: ActErrorImpact::Ignore,
        }
    }

    pub fn degraded(message: String) -> Self {
        Self {
            message,
            impact: ActErrorImpact::Degraded,
        }
    }

    pub fn irrecoverable(message: String) -> Self {
        Self {
            message,
            impact: ActErrorImpact::Irrecoverable,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActErrorImpact {
    /// Continue operating normally.
    Ignore,

    /// Machine can operate, but with reduced capability.
    Degraded,

    /// Machine cannot safely operate.
    Irrecoverable,
}
