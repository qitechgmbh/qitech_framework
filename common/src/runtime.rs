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

        /// subscriber
        subscriber: MachineIdentificationUnique,
    },

    SetMachineConfiguration {
        /// identification of the machine to execute operation on
        ident: MachineIdentificationUnique,

        /// identification of the machine to execute operation on
        path:  String,

        value: ScalarValue,
    },

    InvokeMachineCommand {
        /// identification of the machine to execute operation on
        ident: MachineIdentificationUnique,

        /// resource path e.g. 'puller.start'
        path: String,

        /// command arguments e.g. { ??? }
        data: serde_json::Value,
    },
}

// --- report ---
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RuntimeReport {
    pub created_at: DateTime<Utc>,
    pub responses: Vec<(u64, OperationResult)>,
    pub runtime: RuntimeReportData,
    pub machines: MachinesReport,
    pub logs: Vec<LogRecord>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RuntimeReportData {
    /// list of all events emitted by the runtime itself
    pub state_mutations: Vec<RuntimeStateMutation>,

    /// list of all events emitted by the runtime itself
    pub events: Vec<RuntimeEvent>,
}

// --- event ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeEvent { 
    pub timestamp: DateTime<Utc>,
    pub kind: RuntimeEventKind,
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
    EthercatDeviceInitializationStarted,
    EthercatDeviceInitializationUpdate { state: String },
    EthercatDeviceInitializationFailed { error: String },
    EthercatDeviceInitializationCompleted {
        // devices: Vec<EtherCatDeviceMetaData>
    },

    // --- serial ---
    DiscoveredModbusDevice { path: String },

    // --- machines ---
    MachineConnected { ident: MachineIdentificationUnique },
    MachineDisconnected { ident: MachineIdentificationUnique },
}

// --- state ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStateMutation {
    timestamp: DateTime<Utc>,
    new_state: RuntimeState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeState {
    Started,
    Stopped,
    DicoveringEtherCATInterface,
    InitializingEtherCAT,
    ScanningSerialPorts,
    BuildingMachines,
    Running { in_pre_op: bool },
}
