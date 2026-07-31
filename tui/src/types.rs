use std::collections::HashMap;
use std::ptr;

use indexmap::IndexMap;
use qitech_framework::MachineIdentification;
use qitech_framework::MachineIdentificationUnique;
use qitech_framework::ScalarValue;
use qitech_framework_common::MachineSchema;
use qitech_framework_common::RuntimeStatus;
use qitech_framework_common::schema;
use qitech_framework_common::schema::ConfigPropertyValue;
use qitech_framework_common::schema::MeasurementValue;
use qitech_framework_common::schema::Node;
use qitech_framework_common::schema::NodeKind;
use qitech_framework_common::schema::StatePropertyValue;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Status,
    Content,
}

#[derive(Default)]
pub struct AppState {
    pub rt_status: RuntimeStatus,
    pub schemas: HashMap<MachineIdentification, MachineSchema>,
    pub machines: Vec<MachineEntry>,
}

impl AppState {
    pub fn as_ctx(&self) -> AppContext {
        AppContext {
            rt_status: self.rt_status,
            schemas: ptr::from_ref(&self.schemas),
            machines: ptr::from_ref(self.machines.as_slice()),
        }
    }
}

#[derive(Clone, Copy)]
pub struct AppContext {
    pub rt_status: RuntimeStatus,
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
    pub value: Option<Option<f64>>,
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
                        value: None,
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
