use std::collections::HashMap;
use std::ptr;

use indexmap::IndexMap;
use qitech_framework::MachineIdentification;
use qitech_framework::MachineIdentificationUnique;
use qitech_framework::ScalarValue;
use qitech_framework_common::EtherCATState;
use qitech_framework_common::MachineSchema;
use qitech_framework_common::RuntimeStatus;
use qitech_framework_common::schema;
use qitech_framework_common::schema::ConfigPropertyValue;
use qitech_framework_common::schema::MeasurementValue;
use qitech_framework_common::schema::Node;
use qitech_framework_common::schema::NodeKind;
use qitech_framework_common::schema::StatePropertyValue;

use crate::utils::Timeseries;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Status,
    Content,
}

pub struct AppState {
    pub rt_status: RuntimeStatus,
    pub ecat_status: EtherCATState,
    pub schemas: HashMap<MachineIdentification, MachineSchema>,
    pub machines: Vec<MachineEntry>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            rt_status: RuntimeStatus::Offline,
            ecat_status: EtherCATState::NoInterface,
            schemas: Default::default(),
            machines: Default::default(),
        }
    }

    pub fn as_ctx(&self) -> AppContext {
        AppContext {
            rt_status: self.rt_status,
            ecat_status: self.ecat_status,
            schemas: ptr::from_ref(&self.schemas),
            machines: ptr::from_ref(self.machines.as_slice()),
        }
    }

    pub fn add_machine(&mut self, ident_unique: MachineIdentificationUnique) {
        let ident = ident_unique.identification;

        let Some(schema) = self.schemas.get(&ident) else {
            panic!("NOOOOO: {} | {:?}", ident, &self.schemas);
            return;
        };

        let mut config = IndexMap::new();
        collect_config_fields("", &schema.config_properties, &mut config);

        let mut state = IndexMap::new();
        collect_state_fields("", &schema.state_properties, &mut state);

        let mut measurements = IndexMap::new();
        collect_measurement_fields("", &schema.measurements, &mut measurements);

        let mut commands = IndexMap::new();
        collect_command_fields("", &schema.commands, &mut commands);

        self.machines.push(MachineEntry {
            title: schema.name.clone(),
            ident: ident_unique,
            config,
            state,
            measurements,
            commands,
        });
    }
}

#[derive(Clone, Copy)]
pub struct AppContext {
    pub rt_status: RuntimeStatus,
    pub ecat_status: EtherCATState,
    pub schemas: *const HashMap<MachineIdentification, MachineSchema>,
    pub machines: *const [MachineEntry],
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
        value: String,
    },
}

// --- types ---

pub struct MachineEntry {
    pub title: String,
    pub ident: MachineIdentificationUnique,
    pub config: IndexMap<String, ConfigField>,
    pub state: IndexMap<String, StateField>,
    pub measurements: IndexMap<String, MeasurementField>,
    pub commands: IndexMap<String, CommandField>,
}

pub struct ConfigField {
    pub label: String,
    pub value: Option<ScalarValue>,
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

// --- utils ---
fn collect_config_fields(
    prefix: &str,
    properties: &IndexMap<String, Node<ConfigPropertyValue>>,
    fields: &mut IndexMap<String, ConfigField>,
) {
    for (name, node) in properties {
        let path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}.{name}")
        };

        match &node.kind {
            NodeKind::Branch(children) => {
                collect_config_fields(&path, children, fields);
            }

            NodeKind::Leaf(_) => {
                fields.insert(
                    path.clone(),
                    ConfigField {
                        label: path.clone(),
                        value: None,
                    },
                );
            }
        }
    }
}

fn collect_state_fields(
    prefix: &str,
    properties: &IndexMap<String, Node<StatePropertyValue>>,
    fields: &mut IndexMap<String, StateField>,
) {
    for (name, node) in properties {
        let path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}.{name}")
        };

        match &node.kind {
            NodeKind::Branch(children) => {
                collect_state_fields(&path, children, fields);
            }

            NodeKind::Leaf(_) => {
                fields.insert(
                    path.clone(),
                    StateField {
                        label: path.clone(),
                        value: None,
                    },
                );
            }
        }
    }
}

fn collect_measurement_fields(
    prefix: &str,
    properties: &IndexMap<String, Node<MeasurementValue>>,
    fields: &mut IndexMap<String, MeasurementField>,
) {
    for (name, node) in properties {
        let path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}.{name}")
        };

        match &node.kind {
            NodeKind::Branch(children) => {
                collect_measurement_fields(&path, children, fields);
            }

            NodeKind::Leaf(_) => {
                fields.insert(
                    path.clone(),
                    MeasurementField {
                        label: path.clone(),
                        values: Timeseries::new(4096),
                    },
                );
            }
        }
    }
}

fn collect_command_fields(
    prefix: &str,
    properties: &IndexMap<String, Node<schema::Command>>,
    fields: &mut IndexMap<String, CommandField>,
) {
    for (name, node) in properties {
        let path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}.{name}")
        };

        match &node.kind {
            NodeKind::Branch(children) => {
                collect_command_fields(&path, children, fields);
            }

            NodeKind::Leaf(_) => {
                fields.insert(
                    path.clone(),
                    CommandField {
                        label: path.to_string(),
                        enabled: true,
                    },
                );
            }
        }
    }
}

// ---
