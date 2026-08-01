use std::borrow::Cow;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use soa_derive::StructOfArray;
use thiserror::Error;

use crate::NumericValue;
use crate::ScalarValue;
use crate::ident::MachineIdentificationUnique;
use crate::report::OperationOrigin;
use crate::report::OperationResult;

// --- report ---
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MachinesReport {
    /// machine configuration value mutations
    pub config_value_mutations: Vec<MachineConfigValueMutation>,

    /// machine configuration constraints mutations
    pub config_capability_mutations: Vec<MachineConfigCapabilityMutation>,

    /// machine state mutations
    pub state_mutations: Vec<MachineStateMutation>,

    /// machine measurement snapshot
    pub measurements: MachineMeasurementVec,

    /// machine command invocations
    pub commands: Vec<MachineCommandTrace>,

    /// machine events emitted during this cycle
    pub events: Vec<MachineEmittedEvent>,
}

// --- config ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineConfigValueMutation {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineConfigCapabilityMutation {
    /// target machine
    pub machine: MachineIdentificationUnique,

    /// configuration resource path
    pub path: String,

    /// whether this value may currently be modified
    pub writable: MachineConfigWriteCapability,

    /// current operational constraints
    pub constraints: MachineConfigConstraints,

    /// when constraints changed
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineConfigWriteCapability {
    /// None means writable
    /// Some(reason) means disabled
    pub disabled_reason: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum MachineConfigConstraints {
    #[default]
    None,

    Number {
        min: Option<NumericValue>,
        max: Option<NumericValue>,
    },

    String {
        min_length: Option<usize>,
        max_length: Option<usize>,
        patterns: Vec<String>,
    },

    Enum {
        allowed: Vec<String>,
    },
}

impl MachineConfigConstraints {
    pub fn merged(&self, other: &Self) -> Self {
        match (self, other) {
            (Self::None, other) => other.clone(),

            (this, Self::None) => this.clone(),

            (
                Self::Number { min, max },
                Self::Number {
                    min: other_min,
                    max: other_max,
                },
            ) => Self::Number {
                min: match (min, other_min) {
                    (Some(a), Some(b)) => Some(a.clone().max(b.clone())),
                    (Some(a), None) => Some(a.clone()),
                    (None, Some(b)) => Some(b.clone()),
                    (None, None) => None,
                },
                max: match (max, other_max) {
                    (Some(a), Some(b)) => Some(a.clone().min(b.clone())),
                    (Some(a), None) => Some(a.clone()),
                    (None, Some(b)) => Some(b.clone()),
                    (None, None) => None,
                },
            },

            (
                Self::String {
                    min_length,
                    max_length,
                    patterns,
                },
                Self::String {
                    min_length: other_min_length,
                    max_length: other_max_length,
                    patterns: other_patterns,
                },
            ) => {
                let mut patterns = patterns.clone();
                patterns.extend(other_patterns.clone());

                Self::String {
                    min_length: match (min_length, other_min_length) {
                        (Some(a), Some(b)) => Some((*a).max(*b)),
                        (Some(a), None) => Some(*a),
                        (None, Some(b)) => Some(*b),
                        (None, None) => None,
                    },
                    max_length: match (max_length, other_max_length) {
                        (Some(a), Some(b)) => Some((*a).min(*b)),
                        (Some(a), None) => Some(*a),
                        (None, Some(b)) => Some(*b),
                        (None, None) => None,
                    },
                    patterns,
                }
            }

            (
                Self::Enum { allowed },
                Self::Enum {
                    allowed: other_allowed,
                },
            ) => Self::Enum {
                allowed: allowed
                    .iter()
                    .filter(|v| other_allowed.contains(v))
                    .cloned()
                    .collect(),
            },

            // Different types should not happen after schema validation.
            // Prefer the restrictive side.
            (_, other) => other.clone(),
        }
    }
}

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum MachineConfigWriteError {
    #[error("value out of bounds")]
    OutOfBounds { min: Option<f64>, max: Option<f64> },

    #[error("string does not match required pattern")]
    PatternMismatch { pattern: String },

    #[error("variant is not allowed: {0}")]
    ForbiddenVariant(String),

    #[error("resource is not writable")]
    NotWritable,

    #[error("resource not found")]
    NotFound,
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
pub struct MachineCommandTrace {
    pub request_id: u64,

    /// target machine
    pub target: MachineIdentificationUnique,

    /// command resource path
    pub resource: String,

    /// when the runtime processed the request
    pub timestamp: DateTime<Utc>,

    /// invoke result
    pub result: Result<(), MachineCommandInvokeError>,
}

pub struct MachineCommandCapabilityMutation {
    pub ident: MachineIdentificationUnique,

    /// command resource path
    pub resource: String,

    /// if the command can be executed
    pub can_execute: bool,
}

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum MachineCommandInvokeError {
    #[error("No machine under this identity could be found")]
    NoSuchMachine,

    #[error("command target machine type mismatch: expected `{expected}`, received `{received}`")]
    MachineTypeMismatch { expected: String, received: String },

    #[error("command is disabled")]
    Disabled,

    #[error("command not found")]
    NotFound,

    #[error("command execution failed: {0}")]
    ExecutionError(String),
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
