use std::borrow::Cow;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use soa_derive::StructOfArray;
use crate::types::{Origin, OperationResult, ScalarValue};
use crate::MachineIdentificationUnique;

// --- config ---
#[derive(Debug, Serialize, Deserialize)]
pub struct MachineConfigMutation {
    pub timestamp: DateTime<Utc>,
    pub ident: MachineIdentificationUnique,
    pub name: Cow<'static, str>,
    pub value: ScalarValue,
    pub origin: Origin,
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
    pub origin: Origin,
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
