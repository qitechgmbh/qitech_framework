use std::borrow::Cow;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use soa_derive::StructOfArray;
use crate::MachineIdentificationUnique;
use crate::types::{OperationOrigin, OperationResult, ScalarValue};

// --- report ---
#[derive(Debug, Default, Serialize, Deserialize)]
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
    pub events: Vec<MachineEvent>,
}

// --- config ---
#[derive(Debug, Serialize, Deserialize)]
pub struct MachineConfigMutation {
    /// target machine
    pub target: MachineIdentificationUnique,

    /// configuration resource path (e.g. "laser.power")
    pub resource_path: Cow<'static, str>,

    /// assigned value
    pub value: String,

    /// operation origin
    pub origin: OperationOrigin,

    /// operation result
    pub result: OperationResult,

    /// mutation timestamp
    pub timestamp: DateTime<Utc>,
}

// --- state ---
#[derive(Debug, Serialize, Deserialize)]
pub struct MachineStateMutation {
    /// source machine
    pub source: MachineIdentificationUnique,

    /// state resource path (e.g. "laser.diameter")
    pub resource_path: Cow<'static, str>,

    /// updated value
    pub value: ScalarValue,

    /// mutation timestamp
    pub timestamp: DateTime<Utc>,
}

// --- measurements ---
// --- measurements ---
#[derive(StructOfArray, Debug, Serialize, Deserialize)]
#[soa_derive(Debug, Serialize, Deserialize)]
pub struct MachineMeasurement {
    /// source machine
    pub source: MachineIdentificationUnique,

    /// measurement resource path
    pub resource_path: Cow<'static, str>,

    /// measured value
    pub value: f64,

    /// whether the measurement has no value
    pub is_null: bool,
}

// --- command ---
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
pub struct MachineEvent {
    /// source machine
    pub source: MachineIdentificationUnique,

    /// event resource path
    pub resource_path: Cow<'static, str>,

    /// event payload
    pub data: String,

    /// event timestamp
    pub timestamp: DateTime<Utc>,
}
