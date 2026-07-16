use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use soa_derive::StructOfArray;

mod ident;
pub use ident::MachineIdentification;
pub use ident::MachineIdentificationUnique;

pub mod schema;
mod events;

pub mod vendors {
    include!(concat!(env!("OUT_DIR"), "/vendors.rs"));

    // get_by_id(id: u16) -> Option<u16>

    pub const fn contains_id(id: u16) -> bool {
        get_by_id(id).is_some()
    }

    // get_by_name(name: &str) -> Option<u16>

    pub fn contains_name(name: &str) -> bool {
        get_by_name(name).is_some()
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RuntimeExport {
    /// time when export was created
    pub created_at: DateTime<Utc>,

    /// list of all logs emitted this cycle
    pub logs: Vec<LogRecord>,

    /// list of all events emitted by the runtime itself
    pub runtime_events: Vec<RuntimeEvent>,

    /// list of all events emitted by machines this cycle
    pub machine_events: Vec<MachineEvent>,

    /// list of mutated machine config properties
    pub config_mutations: Vec<ConfigMutationRecord>,

    /// list of mutated machine state properties
    pub state_mutations: Vec<StateMutationRecord>,

    /// snapshot of all measurements of a machine
    pub machine_measurements: Measurements,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeEvent {
    pub timestamp: DateTime<Utc>,
    pub kind: RuntimeEventKind,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RuntimeEventKind {
    MachineConnected(MachineIdentificationUnique),
    MachineDisconnected(MachineIdentificationUnique),
}

impl fmt::Display for RuntimeEventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeEventKind::MachineConnected(id) => {
                write!(f, "machine_connected:{id:?}")
            }
            RuntimeEventKind::MachineDisconnected(id) => {
                write!(f, "machine_disconnected:{id:?}")
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MachineEvent {
    pub timestamp: DateTime<Utc>,
    pub ident: MachineIdentificationUnique,
    pub name: Cow<'static, str>,
    pub data: String, // TODO: use serde_json value
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigMutationRecord {
    pub timestamp: DateTime<Utc>,
    pub ident: MachineIdentificationUnique,
    pub name: Cow<'static, str>,
    pub value: ScalarValue,
    pub origin: ConfigMutationOrigin,
    pub result: ConfigMutationResult,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ConfigMutationOrigin {
    User { request_id: u64 },
    Machine,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(i8)]
pub enum ConfigMutationResult {
    Success = 1,
    OutOfBounds = 2,
    InvalidInput = 3,
}

impl From<ConfigMutationResult> for i8 {
    fn from(v: ConfigMutationResult) -> Self {
        v as i8
    }
}

impl TryFrom<i8> for ConfigMutationResult {
    type Error = String;

    fn try_from(v: i8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Self::Success),
            1 => Ok(Self::OutOfBounds),
            2 => Ok(Self::InvalidInput),
            _ => Err(format!("invalid ConfigMutationOrigin: {v}")),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateMutationRecord {
    pub timestamp: DateTime<Utc>,
    pub ident: MachineIdentificationUnique,
    pub name: Cow<'static, str>,
    pub value: ScalarValue,
}

#[derive(StructOfArray, Debug, Serialize, Deserialize)]
#[soa_derive(Debug, Serialize, Deserialize)]
pub struct MeasurementSnapshot {
    pub ident: MachineIdentificationUnique,
    pub name: String,
    pub value: f64,
    pub null: bool,
}

pub type Measurements = MeasurementSnapshotVec;

#[derive(Debug, Serialize, Deserialize)]
pub struct LogRecord {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub origin: LogOrigin,
    pub message: String,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LogOrigin {
    Machine(MachineIdentificationUnique),
    MainLoop,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(i8)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalarValue {
    Enum(Option<String>),
    String(Option<String>),
    Integer(Option<i64>),
    Float(Option<f64>),
    Boolean(Option<bool>),
}

impl ScalarValue {
    pub fn r#enum(self) -> Option<String> {
        match self {
            ScalarValue::String(value) => value,
            other => panic!("expected Enum, got {:?}", other),
        }
    }

    pub fn string(self) -> Option<String> {
        match self {
            ScalarValue::String(value) => value,
            other => panic!("expected String, got {:?}", other),
        }
    }

    pub fn integer(self) -> Option<i64> {
        match self {
            ScalarValue::Integer(value) => value,
            other => panic!("expected Integer, got {:?}", other),
        }
    }

    pub fn float(self) -> Option<f64> {
        match self {
            ScalarValue::Float(value) => value,
            other => panic!("expected Float, got {:?}", other),
        }
    }

    pub fn boolean(self) -> Option<bool> {
        match self {
            ScalarValue::Boolean(value) => value,
            other => panic!("expected Boolean, got {:?}", other),
        }
    }
}