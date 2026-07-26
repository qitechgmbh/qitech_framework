use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

use crate::LogRecord;
use crate::MachineIdentificationUnique;
use crate::MachinesReport;
use crate::OperationResult;
use crate::ScalarValue;

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

    MachineSubscribe {
        provider: MachineIdentificationUnique,
        consumer: MachineIdentificationUnique,
    },

    MachineUnsubscribe {
        provider: MachineIdentificationUnique,
        consumer: MachineIdentificationUnique,
    },

    SetMachineConfiguration(SetMachineConfigurationRequest),

    InvokeMachineCommand {
        /// target machine
        target: MachineIdentificationUnique,

        /// command resource path
        resource_path: String,

        /// command arguments
        arguments: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetMachineConfigurationRequest {
    /// target machine
    target: MachineIdentificationUnique,

    /// configuration resource path
    resource_path: String,

    /// assigned value
    value: ScalarValue,
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

    /// timings data
    pub timings: TimingsReport,

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
    Terminated {
        error: String,
    },

    // --- ether cat discovery state ---
    EtherCATDiscoveryStarted,
    EtherCATDiscoveryCompleted {
        interface_name: String,
    },

    // --- ether cat initialization ---
    EtherCATDeviceInitializationStarted,
    EtherCATDeviceInitializationUpdate {
        state: String,
    },
    EtherCATDeviceInitializationFailed {
        error: String,
    },
    EtherCATDeviceInitializationCompleted {
        // devices: Vec<EtherCatDeviceMetaData>
    },

    // --- modbus ---
    ModbusDeviceDiscovered {
        path: String,
    },

    // --- machines ---
    MachineConnected {
        ident: MachineIdentificationUnique,
    },

    MachineDisconnected {
        ident: MachineIdentificationUnique,
    },
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

// --- timing ---
#[derive(Debug, Default, Serialize, Deserialize)]
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
        if self.cycle_count == 0 { Duration::ZERO } else { self.duration_peak / self.cycle_count }
    }

    /// Take the current stats and reset for the next export window.
    fn take(&mut self) -> TimingsReport {
        std::mem::take(self)
    }
}