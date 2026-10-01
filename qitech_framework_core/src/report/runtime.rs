use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

use crate::ident::MachineInstanceId;
use crate::report::LogRecord;
use crate::report::MachineResource;
use crate::report::MachinesReport;
use crate::report::TimingsReport;
use crate::request::RuntimeResponse;

// --- report ---
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

impl RuntimeReport {
    pub fn reset(&mut self) {
        self.responses.clear();
        self.timings.reset();
        self.machines.reset();
        self.events.clear();
        self.logs.clear();
    }
}

// --- event ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeEvent {
    // --- ether cat ---
    EtherCATStateUpdate {
        interface: String,
        // state: EtherCATState,
    },

    EtherCATInitializationStarted {
        interface: String,
    },

    EtherCATDeviceInitializationFailed {
        interface: String,
        error: String,
    },

    EtherCATDeviceInitializationCompleted {
        interface: String,
        // devices: Vec<EtherCATDeviceMetadata>,
    },

    AddedMachine {
        ident: MachineInstanceId,
    },

    RemovedMachine {
        ident: MachineInstanceId,
    },

    SubscriptionAdded {
        provider: MachineInstanceId,
        subscriber: MachineInstanceId,
        resources: Vec<MachineResource>,
    },

    SubscriptionRemoved {
        provider: MachineInstanceId,
        subscriber: MachineInstanceId,
    },
}
