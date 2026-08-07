use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

use crate::ScalarValue;
use crate::ScalarValueTypeMismatchError;
use crate::ident::MachineIdentificationUnique;
use crate::report::Constraints;
use crate::report::OperationOrigin;
use crate::report::WriteCapability;
use crate::report::types::ConstraintViolationError;

// --- report ---
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MachinesReport {
    pub config_property_records: Vec<ConfigPropertyRecord>,
    pub state_property_records: Vec<StatePropertyRecord>,
    pub measurement_snapshots: Vec<MeasurementSnapshot>,
    pub command_records: Vec<CommandRecord>,
    pub event_records: Vec<EventRecord>,
}

// --- config ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPropertyRecord {
    pub timestamp: DateTime<Utc>,
    pub machine: MachineIdentificationUnique,
    pub path: String,
    pub event: ConfigPropertyEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigPropertyEvent {
    Registered {
        default: ScalarValue,
        capability: WriteCapability,
        constraints: Constraints,
    },
    DefaultChanged {
        before: ScalarValue,
        after: ScalarValue,
    },
    CapabilityChanged {
        before: WriteCapability,
        after: WriteCapability,
    },
    ConstraintsChanged {
        before: Constraints,
        after: Constraints,
    },
    Written {
        value: ScalarValue,
        origin: OperationOrigin,
        outcome: ConfigPropertyWriteOutcome,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigPropertyWriteOutcome {
    Accepted { changed: bool },
    Rejected(ConfigPropertyWriteError),
}

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ConfigPropertyWriteError {
    #[error("value type mismatch")]
    ValueTypeMismatch(#[from] ScalarValueTypeMismatchError),

    #[error("resource is not writable")]
    NotWritable,

    #[error("value had invalid type")]
    ConstraintViolation(#[from] ConstraintViolationError),
}

// --- state ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatePropertyRecord {
    pub timestamp: DateTime<Utc>,
    pub machine: MachineIdentificationUnique,
    pub path: String,
    pub event: StatePropertyEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatePropertyEvent {
    Registered { value: ScalarValue },
    ValueChanged { value: ScalarValue },
}

// --- measurements ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasurementSnapshot {
    pub machine: MachineIdentificationUnique,
    pub path: String,
    pub value: Option<f64>,
}

// --- command ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRecord {
    pub timestamp: DateTime<Utc>,
    pub machine: MachineIdentificationUnique,
    pub path: String,
    pub event: CommandEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandEvent {
    Registered,
    CapabilityChanged { before: bool, after: bool },
    Invoke(Result<(), CommandInvokeError>),
}

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum CommandInvokeError {
    #[error("command not found")]
    NotFound,

    #[error("command is disabled")]
    Disabled,

    #[error("command execution failed: {0}")]
    ExecutionError(String),
}

// --- event ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    pub timestamp: DateTime<Utc>,
    pub machine: MachineIdentificationUnique,
    pub path: String,
    pub data: Vec<u8>,
}
