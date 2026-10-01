use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

use crate::ScalarValue;
use crate::ident::MachineInstanceId;
use crate::report::ActError;
use crate::report::Constraints;
use crate::report::OperationCapability;
use crate::report::OperationOrigin;

// --- record ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord<T> {
    pub timestamp: DateTime<Utc>,
    pub machine: MachineInstanceId,
    pub path: String,
    pub event: T,
}

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
    ValueChanged {
        value: ScalarValue,
        origin: OperationOrigin,
    },
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
    pub machine: MachineInstanceId,
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
    #[error("command is disabled: {reason}")]
    Disabled { reason: String },

    #[error("command execution failed: {0}")]
    ExecutionError(ActError),
}
