mod run;
use std::collections::HashMap;

use crossterm::event::KeyCode;
use indexmap::IndexMap;
use qitech_framework::MachineIdentification;
use qitech_framework::MachineIdentificationUnique;
use qitech_framework::ScalarValue;
use qitech_framework::runtime::bridge::CrossbeamHandle;
use qitech_framework_common::MachineSchema;
use qitech_framework_common::RuntimeInitEvent;
use qitech_framework_common::RuntimeReport;
use qitech_framework_common::RuntimeRequest;
use qitech_framework_common::RuntimeRequestKind;
use qitech_framework_common::RuntimeStatus;
use qitech_framework_common::schema;
use qitech_framework_common::schema::ConfigPropertyValue;
use qitech_framework_common::schema::MeasurementValue;
use qitech_framework_common::schema::Node;
use qitech_framework_common::schema::NodeKind;
use qitech_framework_common::schema::StatePropertyValue;
use ratatui::Frame;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
pub use run::run;

mod types;
use types::*;

mod utils;

mod widgets;
use widgets::StatusWidget;
use widgets::TabView;
use widgets::WidgetManager;

mod pages;

pub struct App {
    // --- state ---
    focus: Focus,

    rt_status: RuntimeStatus,
    schemas: HashMap<MachineIdentification, MachineSchema>,
    machines: Vec<MachineEntry>,

    // --- widgets ---
    widgets: WidgetManager<AppContext>,
}

impl App {
    #[allow(clippy::new_without_default)]
    pub fn new(schemas: HashMap<MachineIdentification, MachineSchema>) -> Self {
        Self {
            focus: Focus::Status,
            schemas,
            machines: Default::default(),
            rt_status: RuntimeStatus::Offline,
            widgets: WidgetManager::new(vec![
                Box::new(StatusWidget),
                Box::new(ContentWidget::new()),
            ]),
        }
    }

    pub fn update_status(&mut self, value: RuntimeStatus) {
        self.rt_status = value;
    }

    pub fn render(&self, frame: &mut Frame) {
        const TITLE: &str = " QiTech Control (Terminal Edition) ";

        let outer = Block::default().borders(Borders::ALL).title(TITLE);
        frame.render_widget(&outer, frame.area());

        let inner = outer.inner(frame.area());
        self.widgets.render(frame, inner, self.as_context());
    }

    fn as_context(&self) -> AppContext {
        AppContext {
            focus: self.focus,
            rt_status: self.rt_status,
            schemas: &self.schemas as *const HashMap<MachineIdentification, MachineSchema>,
            machines: self.machines.as_slice() as *const [MachineEntry],
        }
    }

    pub fn on_key_event(&mut self, code: KeyCode, handle: &mut CrossbeamHandle) {
        match self.widgets.on_key_event(code, self.as_context()) {
            AppAction::NoAction => {}
            AppAction::SetConfig {
                machine,
                resource,
                value,
            } => {
                handle.send(RuntimeRequest {
                    transaction_id: 0,
                    kind: RuntimeRequestKind::SetMachineConfiguration {
                        target: machine,
                        resource,
                        value,
                    },
                });
            }
        }
    }

    pub fn on_init_event_received<T>(&mut self, event: RuntimeInitEvent<T>) {
        match event {
            RuntimeInitEvent::EtherCATStateUpdate(_) => {}
            RuntimeInitEvent::EtherCATFinalizing => {
                self.rt_status = RuntimeStatus::FinalizingEtherCAT;
            }
            RuntimeInitEvent::EtherCATDiscoveryStarted => {
                self.rt_status = RuntimeStatus::DiscoveringEtherCATInterface;
            }
            RuntimeInitEvent::EtherCATDiscoveryCompleted { .. } => {
                self.rt_status = RuntimeStatus::Initialized;
            }
            RuntimeInitEvent::EtherCATInitializationStarted => {
                self.rt_status = RuntimeStatus::DiscoveringEtherCATInterface;
            }
            RuntimeInitEvent::EtherCATDeviceInitializationFailed { .. } => {
                self.rt_status = RuntimeStatus::DiscoveringEtherCATInterface;
            }
            RuntimeInitEvent::EtherCATDeviceInitializationCompleted { .. } => {
                self.rt_status = RuntimeStatus::DiscoveringEtherCATInterface;
            }
            RuntimeInitEvent::BuildingMachines => {
                self.rt_status = RuntimeStatus::BuildingMachines;
            }
            RuntimeInitEvent::BuiltMachine { ident } => {
                self.add_machine(ident);
            }
            RuntimeInitEvent::FailedToBuildMachine { .. } => {}

            RuntimeInitEvent::ModbusDiscoveryStarted => {
                self.rt_status = RuntimeStatus::DiscoveringModbusDevices;
            }

            _ => {}
        }
    }

    pub fn on_init_complete(&mut self) {
        self.rt_status = RuntimeStatus::Running { in_pre_op: false };
    }

    pub fn on_report_received(&mut self, report: RuntimeReport) {
        let report = report.machines;

        for mutation in &report.config_mutations {
            let Some(entry) = self.find_machine_mut(mutation.machine) else {
                continue;
            };

            let Some(item) = entry.config.get_mut(&mutation.path) else {
                continue;
            };

            item.value = Some(mutation.value.clone());
        }

        for mutation in &report.state_mutations {
            let Some(entry) = self.find_machine_mut(mutation.machine) else {
                continue;
            };

            let Some(item) = entry.state.get_mut(&mutation.path) else {
                continue;
            };

            item.value = Some(mutation.value.clone());
        }

        for measurement in &report.measurements {
            let Some(entry) = self.find_machine_mut(*measurement.machine) else {
                continue;
            };

            let Some(item) = entry.measurements.get_mut(measurement.path) else {
                continue;
            };

            item.value = Some(*measurement.value);
        }
    }

    fn find_machine_mut(
        &mut self,
        ident: MachineIdentificationUnique,
    ) -> Option<&mut MachineEntry> {
        self.machines.iter_mut().find(|m| m.ident == ident)
    }
}

impl App {
    pub fn add_machine(&mut self, ident_unique: MachineIdentificationUnique) {
        let ident = ident_unique.identification;

        let schema = self.schemas.get(&ident).unwrap();

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

    // #[allow(unused)]
    // pub fn remove_machine(&mut self, ident: MachineIdentificationUnique) {
    //     self.machines.retain(|entry| entry.ident != ident);
    // }
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
