use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::{LogRecord, MachineIdentificationUnique, OperationResult, ScalarValue, MachinesReport};

// --- request ---
#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeRequest {
    pub transaction_id: u64,
    pub kind: RuntimeRequestKind,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RuntimeRequestKind {
    WriteMachineDeviceInfo {
        /// machine hardware identification
        machine_ident: MachineIdentificationUnique,

        /// role of the device
        role: u16,

        /// ethercat hardware identification
        subdevice_index: usize,
    },

    RequestMachineSubscription {
        /// source machine
        source: MachineIdentificationUnique,

        /// target subscriber
        subscriber: MachineIdentificationUnique,
    },

    SetMachineConfiguration {
        /// target machine
        target: MachineIdentificationUnique,

        /// configuration resource path
        resource_path: String,

        /// assigned value
        value: ScalarValue,
    },

    InvokeMachineCommand {
        /// target machine
        target: MachineIdentificationUnique,

        /// command resource path
        resource_path: String,

        /// command arguments
        arguments: serde_json::Value,
    }
}

// --- report ---
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RuntimeReport {
    /// report creation timestamp
    pub timestamp: DateTime<Utc>,

    /// results for completed requests
    pub responses: Vec<(u64, OperationResult)>,

    /// runtime activity
    pub runtime: RuntimeReportData,

    /// machine activity
    pub machines: MachinesReport,

    /// runtime log records
    pub logs: Vec<LogRecord>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RuntimeReportData {
    /// runtime state mutations
    pub state_mutations: Vec<RuntimeStateMutation>,

    /// runtime events
    pub events: Vec<RuntimeEvent>,
}

// --- event ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeEvent {
    /// event kind
    pub kind: RuntimeEventKind,

    /// event timestamp
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeEventKind {
    Started,
    Stopped,
    Terminated { error: String },

    // --- ether cat discovery state ---
    EtherCATDiscoveryStarted,
    EtherCATDiscoveryCompleted { interface_name: String },

    // --- ether cat initialization ---
    EtherCATDeviceInitializationStarted,
    EtherCATDeviceInitializationUpdate { state: String },
    EtherCATDeviceInitializationFailed { error: String },
    EtherCATDeviceInitializationCompleted {
        // devices: Vec<EtherCatDeviceMetaData>
    },

    // --- modbus ---
    ModbusDeviceDiscovered { path: String },

    // --- machines ---
    MachineConnected { ident: MachineIdentificationUnique },
    MachineDisconnected { ident: MachineIdentificationUnique },
}

// --- state ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStateMutation {
    /// updated runtime state
    pub state: RuntimeState,

    /// mutation timestamp
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeState {
    Started,
    Stopped,
    DiscoveringEtherCATInterface,
    InitializingEtherCAT,
    ScanningSerialPorts,
    BuildingMachines,
    Running { in_pre_op: bool },
}
