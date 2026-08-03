use std::collections::HashSet;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use soa_derive::StructOfArray;
use thiserror::Error;

use crate::ScalarValue;
use crate::ident::MachineIdentificationUnique;
use crate::report::OperationOrigin;

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
    pub command_traces: Vec<MachineCommandInvokeTrace>,

    /// machine command invocations
    pub command_enabled_mutations: Vec<MachineCommandCapabilityMutation>,

    /// machine events emitted during this cycle
    pub events: Vec<MachineEmittedEvent>,
}

// --- config ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineConfigValueMutation {
    /// target machine
    pub ident: MachineIdentificationUnique,

    /// configuration resource path (e.g. "laser.power")
    pub path: String,

    /// assigned value
    pub value: ScalarValue,

    /// operation origin
    pub origin: OperationOrigin,

    /// operation result
    pub result: MachineConfigWriteResult,

    /// mutation timestamp
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineConfigCapabilityMutation {
    /// target machine
    pub ident: MachineIdentificationUnique,

    /// configuration resource path
    pub path: String,

    /// whether this value may currently be modified
    pub writable: MachineConfigWriteCapability,

    /// current operational constraints
    pub constraints: MachineConfigPropertyConstraints,

    /// when constraints changed
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MachineConfigWriteCapability {
    Allowed,
    Forbidden {
        reason: String,
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum MachineConfigPropertyConstraints {
    #[default]
    None,
    Float {
        min: Option<f64>,
        max: Option<f64>,
    },
    Integer {
        min: Option<i64>,
        max: Option<i64>,
    },
    String {
        min_length: Option<usize>,
        max_length: Option<usize>,
    },
    Enum {
        allowed: Vec<String>,
    },
}

// --- error ---
pub type MachineConfigWriteResult = Result<(), MachineConfigWriteError>;

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
    ResourceNotFound,

    #[error("machine not found")]
    MachineNotFound,

    #[error("machine type mismatch")]
    MachineTypeMismatch,

    #[error("value type mismatch")]
    ValueTypeMismatch,

    #[error("value had invalid type")]
    ConstraintViolation(#[from] ConstraintViolation),
}

#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum ConstraintViolation {
    #[error("types didn't match")]
    TypeMismatch,

    #[error("value {value} is below the minimum {min}")]
    I64BelowMin { value: i64, min: i64 },

    #[error("value {value} is above the maximum {max}")]
    I64AboveMax { value: i64, max: i64 },

    #[error("value {value} is below the minimum {min}")]
    F64BelowMin { value: f64, min: f64 },

    #[error("value {value} is above the maximum {max}")]
    F64AboveMax { value: f64, max: f64 },

    #[error("string length {length} is below the minimum {min}")]
    StringTooShort { length: usize, min: usize },

    #[error("string length {length} is above the maximum {max}")]
    StringTooLong { length: usize, max: usize },

    #[error("string does not match required pattern: {pattern}")]
    PatternMismatch { pattern: String },

    #[error("value {value:?} is not one of the allowed enum variants")]
    VariantForbidden { value: String },
}

// --- state ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineStateMutation {
    /// source machine
    pub ident: MachineIdentificationUnique,

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
    pub ident: MachineIdentificationUnique,

    /// measurement resource path
    pub path: String,

    /// measured value
    pub value: Option<f64>,
}

// --- command ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineCommandInvokeTrace {
    pub request_id: u64,

    /// target machine
    pub ident: MachineIdentificationUnique,

    /// command resource path
    pub resource: String,

    /// when the runtime processed the request
    pub timestamp: DateTime<Utc>,

    /// invoke result
    pub result: Result<(), MachineCommandInvokeError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub ident: MachineIdentificationUnique,

    /// event resource path
    pub path: String,

    /// event payload as json
    pub data: String,

    /// event timestamp
    pub timestamp: DateTime<Utc>,
}
