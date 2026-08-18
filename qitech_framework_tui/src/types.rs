use std::collections::HashMap;
use std::ptr;

use chrono::DateTime;
use chrono::Local;
use crossterm::event::KeyCode;
use indexmap::IndexMap;
use indexmap::IndexSet;
use qitech_framework_core::ScalarValue;
use qitech_framework_core::ident::MachineIdentification;
use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::report::CommandEvent;
use qitech_framework_core::report::ConfigPropertyEvent;
use qitech_framework_core::report::Constraints;
use qitech_framework_core::report::EtherCATStatus;
use qitech_framework_core::report::EventRecord;
use qitech_framework_core::report::OperationCapability;
use qitech_framework_core::report::RuntimeInitStatus;
use qitech_framework_core::report::StatePropertyEvent;
use qitech_framework_core::request::RuntimeRequestError;
use qitech_framework_core::request::RuntimeRequestKind;
use qitech_framework_core::schema::MachineSchema;
use qitech_framework_core::schema::MeasurementKind;
use qitech_framework_core::schema::ScalarPropertyKind;

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
                    records: Default::default(),
                },
            );
        }

        let mut state = IndexMap::new();
        for (name, def) in &schema.state_properties {
            state.insert(
                name.clone(),
                StatePropertyField {
                    label: name.clone(),
                    kind: def.kind.clone(),
                    state: StatePropertyFieldState::NotInitialized,
                    records: Default::default(),
                },
            );
        }

        let mut measurements = IndexMap::new();
        for (name, def) in &schema.measurements {
            measurements.insert(
                name.clone(),
                MeasurementField {
                    label: name.clone(),
                    values: Timeseries::new(4096),
                },
            );

            let stats = match &def.kind {
                MeasurementKind::Boolean => None,
                MeasurementKind::Integer { statistics } => Some(statistics),
                MeasurementKind::Float { statistics, .. } => Some(statistics),
            };

            if let Some(stats) = stats {
                if stats.min {
                    measurements.insert(
                        format!("{}.{}", name.clone(), "min"),
                        MeasurementField {
                            label: format!("{}.{}", name.clone(), "min"),
                            values: Timeseries::new(4096),
                        },
                    );
                }

                if stats.max {
                    measurements.insert(
                        format!("{}.{}", name.clone(), "max"),
                        MeasurementField {
                            label: format!("{}.{}", name.clone(), "max"),
                            values: Timeseries::new(4096),
                        },
                    );
                }

                if stats.avg {
                    measurements.insert(
                        format!("{}.{}", name.clone(), "avg"),
                        MeasurementField {
                            label: format!("{}.{}", name.clone(), "avg"),
                            values: Timeseries::new(4096),
                        },
                    );
                }

                if stats.stddev {
                    measurements.insert(
                        format!("{}.{}", name.clone(), "stddev"),
                        MeasurementField {
                            label: format!("{}.{}", name.clone(), "stddev"),
                            values: Timeseries::new(4096),
                        },
                    );
                }
            }
        }

        let mut commands = IndexMap::new();
        for (name, _) in &schema.commands {
            commands.insert(
                name.clone(),
                CommandField {
                    label: name.clone(),
                    capability: OperationCapability::Allowed,
                    records: Default::default(),
                },
            );
        }

        let mut events = IndexMap::new();
        for (name, _) in &schema.events {
            events.insert(
                name.clone(),
                EventEmitterField {
                    label: name.clone(),
                    records: Default::default(),
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
            events,
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

    pub fn transactions(&self) -> &[Transaction] {
        unsafe { &*self.transactions }
    }

    pub fn schemas(&self) -> &HashMap<MachineIdentification, MachineSchema> {
        unsafe { &*self.schemas }
    }
}

pub enum KeyResult<T> {
    Bubble(KeyCode),
    Handled(T),
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
    pub state: IndexMap<String, StatePropertyField>,
    pub measurements: IndexMap<String, MeasurementField>,
    pub commands: IndexMap<String, CommandField>,
    pub events: IndexMap<String, EventEmitterField>,
    pub subscriptions: IndexSet<MachineIdentificationUnique>,
}

pub struct ConfigField {
    pub kind: ScalarPropertyKind,
    pub label: String,
    pub state: ConfigFieldState,
    pub records: Vec<EventRecord<ConfigPropertyEvent>>,
}

pub enum ConfigFieldState {
    NotInitialized,
    Initialized {
        value: ScalarValue,
        default: ScalarValue,
        capability: OperationCapability,
        constraints: Constraints,
    },
}

pub struct StatePropertyField {
    pub kind: ScalarPropertyKind,
    pub label: String,
    pub state: StatePropertyFieldState,
    pub records: Vec<EventRecord<StatePropertyEvent>>,
}

pub enum StatePropertyFieldState {
    NotInitialized,
    Initialized { value: ScalarValue },
}

pub struct MeasurementField {
    pub label: String,
    pub values: Timeseries,
}

pub struct CommandField {
    pub label: String,
    pub capability: OperationCapability,
    pub records: Vec<EventRecord<CommandEvent>>,
}

pub struct EventEmitterField {
    pub label: String,
    pub records: Vec<EventRecord<String>>,
}

pub struct SubscriptionField {
    pub label: String,
}
