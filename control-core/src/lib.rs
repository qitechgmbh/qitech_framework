use std::borrow::Cow;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use soa_derive::StructOfArray;

mod machine_identification;
pub use machine_identification::MachineIdentification;
pub use machine_identification::MachineIdentificationUnique;

pub mod schema;

#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeExport {
    /// time when export was created
    pub created_at: DateTime<Utc>,

    /// list of all events emitted by the runtime itself
    pub runtime_events: Vec<RuntimeEvent>,

    /// list of all events emitted by machines this cycle
    pub machine_events: Vec<MachineEvent>,

    /// list of mutated machine config properties
    pub config_mutations: Vec<ConfigMutationRecord>,

    /// list of mutated machine state properties
    pub state_mutations: Vec<StateMutationRecord>,

    /// snapshot of all measurements of a machine
    pub measurements: Measurements,

    /// list of all logs emitted this cycle
    pub logs: Vec<LogRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeEvent {
    pub ts: DateTime<Utc>,
    pub name: Cow<'static, str>,
    pub kind: RuntimeEventKind,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RuntimeEventKind {
    MachineConnected(MachineIdentificationUnique),
    MachineDisconnected(MachineIdentificationUnique),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MachineEvent {
    pub ts: DateTime<Utc>,
    pub ident: MachineIdentificationUnique,
    pub name: Cow<'static, str>,
    pub data: Vec<u8>,
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
pub enum ConfigMutationResult {
    Success,
    OutOfBounds,
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
    pub severity: LogLevel,
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
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalarValue {
    String(Option<String>),
    IntegerSigned(Option<i64>),
    IntegerUnsigned(Option<u64>),
    Float(Option<f64>),
    Boolean(Option<bool>),
}