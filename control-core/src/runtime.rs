use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::{LogRecord, MachineCommandCall, MachineConfigMutation, 
    MachineEvent, MachineIdentificationUnique, MachineStateMutation, 
    MachineMeasurementVec, ScalarValue
};

// --- request ---

#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeRequest {
    pub request_id: u64,
    pub kind: RuntimeRequestKind,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RuntimeRequestKind {
    WriteMachineDeviceInfo {
        /// machine hardware identification
        machine_ident: MachineIdentificationUnique,
        role: u16,

        /// ethercat hardware identification
        subdevice_index: usize,
    },
    WriteMachineConfig {
        ident: MachineIdentificationUnique,
        name: String,
        value: ScalarValue,
    },
    ExecuteMachineCommand {
        ident: MachineIdentificationUnique,
        name: String,
        data: serde_json::Value,
    },
}

// --- report ---

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RuntimeReport {
    pub created_at: DateTime<Utc>,
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

// --- event ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeEvent {
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
    DiscoveredModbusDevice { path: String }
}

// --- state ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeStateMutation {
    Started,
    Stopped,
    DicoveringEtherCATInterface,
    InitializingEtherCAT,
    ScanningSerialPorts,
    BuildingMachines,
    Running { in_pre_op: bool },
}
