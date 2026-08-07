use std::mem;
use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

use crate::ident::MachineIdentificationUnique;
use crate::request::RuntimeResponse;

mod types;
pub use types::ConstraintViolationError;
pub use types::Constraints;
pub use types::OperationOrigin;
pub use types::ResourceAccessError;
pub use types::WriteCapability;

mod machines;
pub use machines::CommandEvent;
pub use machines::CommandInvokeError;
pub use machines::CommandRecord;
pub use machines::ConfigPropertyEvent;
pub use machines::ConfigPropertyRecord;
pub use machines::ConfigPropertyWriteError;
pub use machines::ConfigPropertyWriteOutcome;
pub use machines::EventRecord;
pub use machines::MachinesReport;
pub use machines::MeasurementSnapshot;
pub use machines::StatePropertyEvent;
pub use machines::StatePropertyRecord;

pub mod error;

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
    pub responses: Vec<RuntimeResponse>,

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeEvent {
    AddedMachine {
        ident: MachineIdentificationUnique,
    },

    RemovedMachine {
        ident: MachineIdentificationUnique,
    },

    SubscriptionAdded {
        provider: MachineIdentificationUnique,
        subscriber: MachineIdentificationUnique,
        resources: Vec<MachineResource>,
    },

    SubscriptionRemoved {
        provider: MachineIdentificationUnique,
        subscriber: MachineIdentificationUnique,
    },
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

// --- subscription ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineResource {
    path: String,
    kind: ResourceKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceKind {
    ConfigProperty,
    StateProperty,
    Measurement,
    Command,
    Event,
}
