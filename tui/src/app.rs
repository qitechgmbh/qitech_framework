use std::collections::HashMap;

use crossterm::event::KeyCode;
use crossterm::event::MouseEvent;
use crossterm::event::MouseEventKind;
use indexmap::IndexMap;
use qitech_framework::MachineIdentification;
use qitech_framework::MachineIdentificationUnique;
use qitech_framework::ScalarValue;
use qitech_framework::runtime::bridge::CrossbeamHandle;

use qitech_framework::runtime::bridge::CrossbeamRuntimeInitEvent;
use qitech_framework_common::MachineMeasurement;
use qitech_framework_common::MachineSchema;
use qitech_framework_common::RuntimeReport;
use qitech_framework_common::schema::ConfigPropertyValue;
use qitech_framework_common::schema::MeasurementValue;
use qitech_framework_common::schema::Node;
use qitech_framework_common::schema::NodeKind;
use qitech_framework_common::schema::StatePropertyValue;

pub enum RuntimeStatus {
    Offline,
    Starting,
    Running,
}

pub struct App {
    pub runtime_status: RuntimeStatus,
    pub schemas: HashMap<MachineIdentification, MachineSchema>,
    pub pages: HashMap<MachineIdentificationUnique, MachinePage>,

    // --- positions ---
    pub page_pos: usize,
    pub item_pos: usize,
    pub highlight: bool,

    // --- misc ---
    pub running: bool,
}

impl App {
    pub fn new(schemas: HashMap<MachineIdentification, MachineSchema>) -> Self {
        Self {
            running: true,
            schemas,
            pages: Default::default(),
            page_pos: 0,
            item_pos: 0,
            highlight: false,
            runtime_status: RuntimeStatus::Offline,
        }
    }

    pub fn running(&self) -> bool {
        self.running
    }

    pub fn handle_key(&mut self, key: KeyCode, handle: &mut CrossbeamHandle) {
        match key {
            KeyCode::Char('q') => self.running = false,
            KeyCode::Esc => self.highlight = false,
            KeyCode::Left => {
                self.page_pos = self.page_pos.saturating_sub(1);
            }
            KeyCode::Right => {
                let max = self.pages.len().saturating_sub(1);
                self.page_pos = (self.page_pos + 1).min(max);
            }
            _ => {}
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent, handle: &mut CrossbeamHandle) {
        if let MouseEventKind::Down(_) = mouse.kind {
            // let row = mouse.row as usize;

            // // select config property
            // if row > 1 {
            //     let index = row - 2;

            //     if index < self.model.config.len() {
            //         self.pointer = index;
            //         self.pointer_enabled = true;
            //     }
            // }
        }
    }

    pub fn handle_init_event(&mut self, event: CrossbeamRuntimeInitEvent) {
        match event {
            CrossbeamRuntimeInitEvent::EtherCATStateUpdate(_) => {

            },
            CrossbeamRuntimeInitEvent::EtherCATFinalizing => {

            },
            CrossbeamRuntimeInitEvent::EtherCATDiscoveryStarted => {

            },
            CrossbeamRuntimeInitEvent::EtherCATDiscoveryCompleted { .. } => {

            },
            CrossbeamRuntimeInitEvent::EtherCATInitializationStarted => {

            },
            CrossbeamRuntimeInitEvent::EtherCATDeviceInitializationFailed { .. } => {

            },
            CrossbeamRuntimeInitEvent::EtherCATDeviceInitializationCompleted { .. } => {

            },
            CrossbeamRuntimeInitEvent::BuildingMachines => {
                
            },
            CrossbeamRuntimeInitEvent::BuiltMachine { ident: ident_unique } => {
                let ident = ident_unique.identification;

                let schema = self.schemas.get(&ident).unwrap();

                let mut config = IndexMap::new();
                collect_config_fields("", &schema.config_properties, &mut config);

                let mut state = IndexMap::new();
                collect_state_fields("", &schema.state_properties, &mut state);

                let mut measurements = IndexMap::new();
                collect_measurement_fields("", &schema.measurements, &mut measurements);

                self.pages.insert(ident_unique, MachinePage { 
                    name: schema.name.clone(), 
                    serial: ident_unique.serial,
                    config, 
                    state, 
                    measurements,
                });
            },
            CrossbeamRuntimeInitEvent::FailedToBuildMachine { .. } => {},
            _ => {},
        }
    }

    pub fn handle_report(&mut self, report: RuntimeReport) {

        for mutation in &report.machines.config_mutations {
            let entry = self.pages.get_mut(&mutation.machine).expect("msg");
            let item = entry.config.get_mut(&mutation.path).unwrap();
            
            item.value = mutation.value.clone();
            item.empty = false;
        }

        for mutation in &report.machines.state_mutations {
            let entry = self.pages.get_mut(&mutation.machine).expect("msg");
            let item = entry.state.get_mut(&mutation.path).unwrap();
            
            item.value = mutation.value.clone();
            item.empty = false;
        }

        for measurement in &report.machines.measurements {
            let entry = self.pages.get_mut(measurement.machine).expect("msg");
            let item = entry.measurements.get_mut(measurement.path).unwrap();
            
            item.value = *measurement.value;
            item.empty = false;
        }

        _ = report;
    }
}

// --- types ---
pub struct MachinePage {
    pub name: String,
    pub serial: u16,
    pub config: IndexMap<String, ConfigField>,
    pub state: IndexMap<String, StateField>,
    pub measurements: IndexMap<String, MeasurementField>,
}

pub struct ConfigField {
    pub label: String,
    pub value: ScalarValue,
    pub empty: bool,
}

pub struct StateField {
    pub label: String,
    pub value: ScalarValue,
    pub empty: bool,
}

pub struct MeasurementField {
    pub label: String,
    pub value: Option<f64>,
    pub empty: bool,
}

// --- x ---
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
                fields.insert(path.clone(), ConfigField {
                    label: path.clone(),
                    value: ScalarValue::Float(None),
                    empty: true,
                });
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
                fields.insert(path.clone(), StateField {
                    label: path.clone(),
                    value: ScalarValue::Float(None),
                    empty: true,
                });
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
                    value: Some(1.0),
                    empty: true,
                });
            }
        }
    }
}
