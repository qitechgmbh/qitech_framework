use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

use crate::DeviceIdentification;
use crate::LogRecord;
use crate::MachineIdentificationUnique;
use crate::MachinesReport;

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

    SetMachineConfiguration {
        /// target machine
        target: MachineIdentificationUnique,

        /// resource path
        resource: String,

        /// value to write
        value: String,
    },

    InvokeMachineCommand {
        /// target machine
        target: MachineIdentificationUnique,

        /// command resource path
        resource: String,

        /// command arguments
        arguments: String,
    },

    MachineSubscribe {
        provider: MachineIdentificationUnique,
        subscriber: MachineIdentificationUnique,
    },

    MachineUnsubscribe {
        provider: MachineIdentificationUnique,
        subscriber: MachineIdentificationUnique,
    },
}

// --- report ---
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuntimeReport {
    /// report creation timestamp
    pub timestamp: DateTime<Utc>,

    /// results for completed requests
    pub responses: Vec<(u64, Result<(), String>)>,

    /// timings data
    pub timings: TimingsReport,

    /// machine activity
    pub machines: MachinesReport,

    /// runtime events
    pub events: Vec<RuntimeEvent>,

    /// runtime log records
    pub logs: Vec<LogRecord>,
}

// --- event ---
#[derive(Debug, Clone)]
pub enum RuntimeInitEvent {
    EtherCATStateUpdate(EtherCATState),
    EtherCATFinalizing,

    // --- ether cat discovery ---
    EtherCATDiscoveryStarted,
    EtherCATDiscoveryCompleted {
        interface: String,
    },

    // --- ether cat device ---
    EtherCATInitializationStarted,
    EtherCATDeviceInitializationFailed {
        error: String,
    },
    EtherCATDeviceInitializationCompleted {
        devices: Vec<EtherCATDeviceMetadata>,
    },

    // --- modbus rtu ---
    ModbusDiscoveryStarted,

    // --- machine ---
    BuildingMachines,
    BuiltMachine {
        ident: MachineIdentificationUnique,
    },
    FailedToBuildMachine {
        ident: MachineIdentificationUnique,
    },

    // --- finished ---
    Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeEvent {
    AddedMachine { ident: MachineIdentificationUnique },
    RemovedMachine { ident: MachineIdentificationUnique },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EtherCATState {
    NoInterface,
    Boot,
    Init,
    PreOp,
    PreopPdi,
    Op,
}

// --- state ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStateMutation {
    /// updated runtime state
    pub state: RuntimeStatus,

    /// mutation timestamp
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RuntimeStatus {
    Offline,
    DiscoveringEtherCATInterface,
    InitializingEtherCAT,
    DiscoveringModbusDevices,
    BuildingMachines,
    FinalizingEtherCAT,
    Initialized,
    Running { in_pre_op: bool },
}

// --- timing ---
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimingsReport {
    // total number of cycles
    cycle_count: u32,

    /// total duration spent executing
    duration_total: Duration,

    /// duration of the longest cycle
    duration_peak: Duration,

    // cycles that exceeded cycle_timeout
    overrun_count: u32,
}

impl TimingsReport {
    fn record(&mut self, duration: Duration, budget: Duration) {
        self.cycle_count += 1;
        self.duration_total += duration;
        if duration > self.duration_peak {
            self.duration_peak = duration;
        }

        if duration > budget {
            self.overrun_count += 1;
        }
    }

    fn duration_avg(&self) -> Duration {
        if self.cycle_count == 0 {
            Duration::ZERO
        } else {
            self.duration_peak / self.cycle_count
        }
    }

    /// Take the current stats and reset for the next export window.
    fn take(&mut self) -> TimingsReport {
        std::mem::take(self)
    }
}

// --- stats ---
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatsReport {
    /// Number of recorded machine configuration property mutations.
    pub recorded_machine_config_property_mutations: u32,

    /// Number of recorded machine state property mutations.
    pub recorded_machine_state_property_mutations: u32,

    /// Number of machine events emitted.
    pub emitted_machine_events: u32,

    /// Number of API requests processed.
    pub processed_requests: u32,
}

// --- misc ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtherCATDeviceMetadata {
    pub configured_address: u16,
    pub name: String,
    pub vendor_id: u32,
    pub product_id: u32,
    pub revision: u32,
    pub device_identification: DeviceIdentification,
}
