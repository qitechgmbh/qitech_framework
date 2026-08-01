use std::borrow::Cow;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use soa_derive::StructOfArray;

use crate::ScalarValue;
use crate::ident::MachineIdentificationUnique;
use crate::report::OperationOrigin;
use crate::report::OperationResult;

// --- report ---
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MachinesReport {
    /// machine configuration mutations
    pub config_mutations: Vec<MachineConfigMutation>,

    /// machine state mutations
    pub state_mutations: Vec<MachineStateMutation>,

    /// machine measurement snapshot
    pub measurements: MachineMeasurementVec,

    /// machine command invocations
    pub commands: Vec<MachineCommandCall>,

    /// machine events emitted during this cycle
    pub events: Vec<MachineEmittedEvent>,
}

// --- config ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineConfigMutation {
    /// target machine
    pub machine: MachineIdentificationUnique,

    /// configuration resource path (e.g. "laser.power")
    pub path: String,

    /// assigned value
    pub value: ScalarValue,

    /// operation origin
    pub origin: OperationOrigin,

    /// operation result
    pub result: OperationResult,

    /// mutation timestamp
    pub timestamp: DateTime<Utc>,
}

// --- state ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineStateMutation {
    /// source machine
    pub machine: MachineIdentificationUnique,

    /// state resource path (e.g. "laser.diameter")
    pub path: String,

    /// updated value
    pub value: ScalarValue,

    /// mutation timestamp
    pub timestamp: DateTime<Utc>,
}

// --- measurements ---
#[derive(StructOfArray, Debug, Serialize, Deserialize)]
#[soa_derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineMeasurement {
    /// source machine
    pub machine: MachineIdentificationUnique,

    /// measurement resource path
    pub path: String,

    /// measured value
    pub value: Option<f64>,
}

// --- command ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineCommandCall {
    /// target machine
    pub target: MachineIdentificationUnique,

    /// command resource path
    pub resource_path: Cow<'static, str>,

    /// command arguments
    pub arguments: String,

    /// command timestamp
    pub timestamp: DateTime<Utc>,

    /// execution result
    pub result: OperationResult,
}

// --- event ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineEmittedEvent {
    /// source machine
    pub machine: MachineIdentificationUnique,

    /// event resource path
    pub path: Cow<'static, str>,

    /// event payload
    pub data: String,

    /// event timestamp
    pub timestamp: DateTime<Utc>,
}
