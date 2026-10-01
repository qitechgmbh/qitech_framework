use core::fmt;

use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

// --- base ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineResource {
    path: String,
    kind: MachineResourceKind,
}

// --- kind ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MachineResourceKind {
    ConfigProperty,
    StateProperty,
    Measurement,
    Command,
    Event,
}

impl fmt::Display for MachineResourceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MachineResourceKind::ConfigProperty => write!(f, "ConfigProperty"),
            MachineResourceKind::StateProperty => write!(f, "StateProperty"),
            MachineResourceKind::Measurement => write!(f, "Measurement"),
            MachineResourceKind::Command => write!(f, "Command"),
            MachineResourceKind::Event => write!(f, "Event"),
        }
    }
}

// --- error ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Error)]
pub enum MachineResourceAccessError {
    #[error("machine not found")]
    MachineNotFound,

    #[error("resource not found: {kind} at '{path}'")]
    ResourceNotFound { kind: MachineResourceKind, path: String },

    #[error("resource type mismatch: expected {expected}, received {actual}")]
    TypeMismatch { expected: String, actual: String },
}
