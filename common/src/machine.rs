use std::borrow::Cow;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use soa_derive::StructOfArray;
use crate::types::{OperationOrigin, OperationResult, ScalarValue};
use crate::MachineIdentificationUnique;

// --- report ---
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MachinesReport {
    /// list of mutated machine config properties
    pub config_mutations: Vec<MachineConfigMutation>,

    /// list of mutated machine state properties
    pub state_mutations: Vec<MachineStateMutation>,

    /// snapshot of all measurements of a machine
    pub measurements: MachineMeasurementVec,

    /// list of all invoked commands
    pub commands: Vec<MachineCommandCall>,

    /// list of all events emitted by machines this cycle
    pub events: Vec<MachineEvent>,
}

// --- config ---
#[derive(Debug, Serialize, Deserialize)]
pub struct MachineConfigMutation {
    pub timestamp: DateTime<Utc>,
    pub ident: MachineIdentificationUnique,
    pub name: Cow<'static, str>,
    pub value: ScalarValue,
    pub origin: OperationOrigin,
    pub result: OperationResult,
}

// --- state ---
#[derive(Debug, Serialize, Deserialize)]
pub struct MachineStateMutation {
    pub timestamp: DateTime<Utc>,
    pub ident: MachineIdentificationUnique,
    pub name: Cow<'static, str>,
    pub value: ScalarValue,
}

// --- measurements ---
#[derive(StructOfArray, Debug, Serialize, Deserialize)]
#[soa_derive(Debug, Serialize, Deserialize)]
pub struct MachineMeasurement {
    pub ident: MachineIdentificationUnique,
    pub name: String,
    pub value: f64,
    pub null: bool,
}

// --- command ---
#[derive(Debug, Serialize, Deserialize)]
pub struct MachineCommandCall {
    pub timestamp: DateTime<Utc>,
    pub ident: MachineIdentificationUnique,
    pub name: Cow<'static, str>,
    pub data: String,
    pub origin: OperationOrigin,
    pub result: Result<(), String>,
}

// --- event ---
#[derive(Debug, Serialize, Deserialize)]
pub struct MachineEvent {
    pub timestamp: DateTime<Utc>,
    pub ident: MachineIdentificationUnique,
    pub name: Cow<'static, str>,
    pub data: serde_json::Value,
}
