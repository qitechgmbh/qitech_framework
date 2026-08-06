use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use soa_derive::StructOfArray;
use thiserror::Error;

use crate::NumericValue;
use crate::ScalarValue;
use crate::ScalarValueTypeMismatchError;
use crate::ident::MachineIdentificationUnique;
use crate::report::OperationOrigin;

// --- report ---
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MachinesReport {
    /// machine configuration value records
    pub config_property_write_records: Vec<ConfigPropertyValueRecord>,

    /// machine configuration state records
    pub config_property_state_records: Vec<ConfigPropertyStateRecord>,

    /// machine state mutations
    pub state_property_write_records: Vec<StatePropertyWriteRecord>,

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
pub struct ConfigPropertyValueRecord {
    /// target machine
    pub ident: MachineIdentificationUnique,

    /// configuration resource path (e.g. "laser.power")
    pub path: String,

    /// assigned value
    pub value: ScalarValue,

    /// operation origin
    pub origin: OperationOrigin,

    /// operation result
    pub result: ConfigPropertyWriteResult,

    /// mutation timestamp
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPropertyStateRecord {
    /// target machine
    pub ident: MachineIdentificationUnique,

    /// configuration resource path
    pub path: String,

    pub kind: ConfigPropertyStateChange,

    /// when state changed
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigPropertyStateChange {
    WriteCapability(WriteCapability),
    Constraints(ParameterConstraints),
    DefaultValue(ScalarValue),
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum WriteCapability {
    #[default]
    Allowed,
    Forbidden {
        reason: String,
    },
}

impl WriteCapability {
    pub const fn is_allowed(&self) -> bool {
        matches!(self, WriteCapability::Allowed)
    }

    pub const fn forbidden(&self) -> bool {
        matches!(self, WriteCapability::Forbidden { .. })
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterConstraints {
    #[default]
    None,
    Numeric {
        min: NumericValue,
        max: NumericValue,
        nullable: bool,
    },
    String {
        min_length: Option<usize>,
        max_length: Option<usize>,
        pattern: Option<String>,
        nullable: bool,
    },
    Enum {
        allowed: Vec<String>,
        nullable: bool,
    },
}

// --- error ---
pub type ConfigPropertyWriteResult = Result<(), ConfigPropertyWriteError>;

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ConfigPropertyWriteError {
    #[error("resource not found")]
    NotFound,

    #[error("value type mismatch")]
    ValueTypeMismatch(#[from] ScalarValueTypeMismatchError),

    #[error("resource is not writable")]
    NotWritable,

    #[error("value had invalid type")]
    ConstraintViolation(#[from] ConstraintViolation),
}

#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum ConstraintViolation {
    #[error("types didn't match")]
    TypeMismatch,

    #[error("value {value} is below the minimum {min}")]
    BelowMin {
        value: NumericValue,
        min: NumericValue,
    },

    #[error("value {value} is above the maximum {max}")]
    AboveMax {
        value: NumericValue,
        max: NumericValue,
    },

    #[error("value {value} cannot be null")]
    CannotBeNull { value: ScalarValue },

    #[error("string length {length} is below the minimum {min}")]
    StringTooShort { length: usize, min: usize },

    #[error("string length {length} is above the maximum {max}")]
    StringTooLong { length: usize, max: usize },

    #[error("string does not match required pattern: {pattern}")]
    PatternMismatch { pattern: String },

    #[error("value {value:?} is not one of the allowed enum variants")]
    ForbiddenVariant { value: String },
}

// --- state ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatePropertyWriteRecord {
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
    #[error("command not found")]
    NotFound,

    #[error("command is disabled")]
    Disabled,

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
