use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

use crate::ScalarValue;
use crate::ScalarValueTypeMismatchError;
use crate::ident::MachineIdentificationUnique;
use crate::report::Constraints;
use crate::report::EventRecord;
use crate::report::OperationCapability;
use crate::report::OperationOrigin;
use crate::report::error::ActError;
use crate::report::types::ConstraintViolationError;

// --- report ---
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MachinesReport {
    pub config_property_records: Vec<EventRecord<ConfigPropertyEvent>>,
    pub state_property_records: Vec<EventRecord<StatePropertyEvent>>,
    pub measurement_snapshots: Vec<MeasurementSnapshot>,
    pub command_records: Vec<EventRecord<CommandEvent>>,
    pub event_records: Vec<EventRecord<String>>,
}

impl MachinesReport {
    pub fn reset(&mut self) {
        self.config_property_records.clear();
        self.state_property_records.clear();
        self.measurement_snapshots.clear();
        self.command_records.clear();
        self.event_records.clear();
    }
}

// --- config ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigPropertyEvent {
    Registered {
        default: ScalarValue,
        capability: OperationCapability,
        constraints: Constraints,
    },
    DefaultChanged(ScalarValue),
    CapabilityChanged(OperationCapability),
    ConstraintsChanged(Constraints),
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
    #[error("value type does not match the expected type")]
    ValueTypeMismatch(#[from] ScalarValueTypeMismatchError),

    #[error("resource is not writable")]
    NotWritable,

    #[error("value violates the resource constraints")]
    ConstraintViolation(#[from] ConstraintViolationError),
}

// --- state ---
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
pub enum CommandEvent {
    Registered,
    CapabilityChanged(OperationCapability),
    Executed(Result<(), CommandExecuteError>),
}

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum CommandExecuteError {
    #[error("command is disabled")]
    Disabled { reason: String },

    #[error("command execution failed: {0}")]
    ExecutionError(ActError),
}
