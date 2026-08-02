use std::mem;
use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

use crate::ident::MachineIdentificationUnique;

mod machines;
pub use machines::MachineCommandCapabilityMutation;
pub use machines::MachineCommandInvokeError;
pub use machines::MachineCommandInvokeTrace;
pub use machines::MachineConfigCapabilityMutation;
pub use machines::MachineConfigPropertyConstraints;
pub use machines::MachineConfigValueMutation;
pub use machines::MachineConfigWriteCapability;
pub use machines::MachineConfigWriteError;
pub use machines::MachineEmittedEvent;
pub use machines::MachineMeasurement;
pub use machines::MachineStateMutation;
pub use machines::MachinesReport;

mod logs;
pub use logs::LogLevel;
pub use logs::LogRecord;
pub use logs::LogSource;

mod init;
pub use init::EtherCATDeviceMetadata;
pub use init::EtherCATStatus;
pub use init::RuntimeInitEvent;
pub use init::RuntimeInitStatus;

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
    pub events: Vec<RuntimeRunEvent>,

    /// runtime log records
    pub logs: Vec<LogRecord>,
}

// --- event ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeRunEvent {
    AddedMachine { ident: MachineIdentificationUnique },
    RemovedMachine { ident: MachineIdentificationUnique },
}

// --- response ---
pub struct RequestResults {}

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
    pub fn record(&mut self, duration: Duration, budget: Duration) {
        self.cycle_count += 1;
        self.duration_total += duration;
        if duration > self.duration_peak {
            self.duration_peak = duration;
        }

        if duration > budget {
            self.overrun_count += 1;
        }
    }

    pub fn duration_avg(&self) -> Duration {
        if self.cycle_count == 0 {
            Duration::ZERO
        } else {
            self.duration_peak / self.cycle_count
        }
    }

    /// Take the current stats and reset for the next export window.
    pub fn take(&mut self) -> TimingsReport {
        mem::take(self)
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OperationOrigin {
    Request { request_id: u64 },
    Machine,
}

impl From<OperationOrigin> for u64 {
    fn from(value: OperationOrigin) -> Self {
        match value {
            OperationOrigin::Request { request_id } => request_id,
            OperationOrigin::Machine => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(i8)]
pub enum OperationResult {
    Success = 0,
    Failure = 1,
}

impl From<OperationResult> for i8 {
    fn from(v: OperationResult) -> Self {
        v as i8
    }
}

impl TryFrom<i8> for OperationResult {
    type Error = String;

    fn try_from(v: i8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Self::Success),
            1 => Ok(Self::Failure),
            _ => Err(format!("invalid ConfigMutationOrigin: {v}")),
        }
    }
}
