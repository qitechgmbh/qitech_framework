use std::collections::HashMap;
use std::ptr;

use chrono::DateTime;
use chrono::Local;
use chrono::Utc;
use indexmap::IndexMap;
use indexmap::IndexSet;
use qitech_framework_core::ScalarValue;
use qitech_framework_core::ident::MachineIdentification;
use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::report::Constraints;
use qitech_framework_core::report::EtherCATStatus;
use qitech_framework_core::report::RuntimeInitStatus;
use qitech_framework_core::report::WriteCapability;
use qitech_framework_core::request::RuntimeRequest;
use qitech_framework_core::request::RuntimeRequestError;
use qitech_framework_core::request::RuntimeRequestKind;
use qitech_framework_core::schema::ConfigPropertyKind;
use qitech_framework_core::schema::MachineSchema;

use crate::utils::Timeseries;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RuntimeStatus {
    Offline,
    Initializing(RuntimeInitStatus),
    Running,
    Disconnected,
}

pub struct AppState {
    pub rt_status: RuntimeStatus,
    pub ecat_status: EtherCATStatus,
    pub schemas: HashMap<MachineIdentification, MachineSchema>,
    pub machines: Vec<MachineEntry>,
    pub transactions: Vec<Transaction>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            rt_status: RuntimeStatus::Offline,
            ecat_status: EtherCATStatus::NoInterface,
            schemas: Default::default(),
            machines: Default::default(),
            transactions: Default::default(),
        }
    }

    pub fn as_ctx(&self) -> AppContext {
        AppContext {
            rt_status: self.rt_status,
            ecat_status: self.ecat_status,
            schemas: ptr::from_ref(&self.schemas),
            machines: ptr::from_ref(self.machines.as_slice()),
            transactions: ptr::from_ref(self.transactions.as_slice()),
        }
    }

    pub fn add_machine(&mut self, ident_unique: MachineIdentificationUnique) {
        let ident = ident_unique.identification;

        let Some(schema) = self.schemas.get(&ident) else {
            return;
        };

        let mut config = IndexMap::new();
        for (name, def) in &schema.config_properties {
            config.insert(
                name.clone(),
                ConfigField {
                    kind: def.kind.clone(),
                    label: name.clone(),
                    state: ConfigFieldState::NotInitialized,
                },
            );
        }

        let mut state = IndexMap::new();
        for (name, _) in &schema.state_properties {
            state.insert(
                name.clone(),
                StateField {
                    label: name.clone(),
                    value: None,
                },
            );
        }

        let mut measurements = IndexMap::new();
        for (name, _) in &schema.measurements {
            measurements.insert(
                name.clone(),
                MeasurementField {
                    label: name.clone(),
                    values: Timeseries::new(4096),
                },
            );
        }

        let mut commands = IndexMap::new();
        for (name, _) in &schema.commands {
            commands.insert(
                name.clone(),
                CommandField {
                    label: name.clone(),
                    enabled: true,
                },
            );
        }

        self.machines.push(MachineEntry {
            title: schema.name.clone(),
            ident: ident_unique,
            config,
            state,
            measurements,
            commands,
            subscriptions: IndexSet::new(),
        });
    }
}

#[derive(Clone, Copy)]
pub struct AppContext {
    pub rt_status: RuntimeStatus,
    pub ecat_status: EtherCATStatus,
    pub schemas: *const HashMap<MachineIdentification, MachineSchema>,
    pub machines: *const [MachineEntry],
    pub transactions: *const [Transaction],
}

impl AppContext {
    pub fn machines(&self) -> &[MachineEntry] {
        unsafe { &*self.machines }
    }
}

pub enum AppAction {
    NoAction,
    SetConfig {
        machine: MachineIdentificationUnique,
        resource: String,
        value: ScalarValue,
    },
    ExecuteCommand {
        machine: MachineIdentificationUnique,
        resource: String,
    },
    Subscribe {
        provider: MachineIdentificationUnique,
        subscriber: MachineIdentificationUnique,
    },
    Unsubscribe {
        provider: MachineIdentificationUnique,
        subscriber: MachineIdentificationUnique,
    },
}

// --- types ---
#[derive(Debug)]
pub struct Transaction {
    pub id: u64,
    pub timestamp: DateTime<Local>,
    pub request: RuntimeRequestKind,
    pub result: Result<(), RuntimeRequestError>,
}

pub struct MachineEntry {
    pub title: String,
    pub ident: MachineIdentificationUnique,
    pub config: IndexMap<String, ConfigField>,
    pub state: IndexMap<String, StateField>,
    pub measurements: IndexMap<String, MeasurementField>,
    pub commands: IndexMap<String, CommandField>,
    pub subscriptions: IndexSet<MachineIdentificationUnique>,
}

pub struct ConfigField {
    pub kind: ConfigPropertyKind,
    pub label: String,
    pub state: ConfigFieldState,
}

pub enum ConfigFieldState {
    NotInitialized,
    Initialized {
        value: ScalarValue,
        default: ScalarValue,
        capability: WriteCapability,
        constraints: Constraints,
    },
}

pub struct StateField {
    pub label: String,
    pub value: Option<ScalarValue>,
}

pub struct MeasurementField {
    pub label: String,
    pub values: Timeseries,
}

pub struct CommandField {
    pub label: String,
    pub enabled: bool,
}

pub struct SubscriptionField {
    pub label: String,
}
