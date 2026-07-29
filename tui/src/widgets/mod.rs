use std::collections::HashMap;

use indexmap::IndexMap;
use qitech_framework::MachineIdentification;
use qitech_framework::MachineIdentificationUnique;
use qitech_framework::ScalarValue;
use qitech_framework_common::MachineSchema;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::app::RuntimeStatus;

mod status;
pub use status::StatusWidget;

mod menu;
pub use menu::MenuWidget;

mod content_root;
pub use content_root::ContentRoot;

mod machines;


pub enum Menu {
    Machines,
    EtherCAT,
    Modbus,
    Logs,
}

impl Menu {
    pub fn next(self) -> Self {
        match self {
            Menu::Machines => Menu::EtherCAT,
            Menu::EtherCAT => Menu::Modbus,
            Menu::Modbus => Menu::Logs,
            Menu::Logs => Menu::Machines,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Menu::Machines => Menu::Logs,
            Menu::EtherCAT => Menu::Machines,
            Menu::Modbus => Menu::EtherCAT,
            Menu::Logs => Menu::Modbus,
        }
    }
}

pub struct AppContext {
    // --- positions ---
    selected_menu: Menu,

    // --- misc ---
    runtime_status: RuntimeStatus,
    schemas: HashMap<MachineIdentification, MachineSchema>,
    machines: Vec<MachineEntry>,
}

pub struct Styles {
    editing: Style,
    selected: Style,
}

pub trait AppWidget<T> {
    fn height(&self) -> Constraint;
    fn display(&self, ctx: &T, frame: &mut Frame);
}

#[derive(Clone, Copy)]
pub enum AppWidgetState {
    NoFocus,
    InFocus,
    Editing,
}

// --- types ---
pub struct MachineEntry {
    pub title: String,
    pub ident: MachineIdentificationUnique,
    pub config: IndexMap<String, ScalarValueField>,
    pub state: IndexMap<String, ScalarValueField>,
    pub measurements: IndexMap<String, MeasurementField>,
}

pub struct ScalarValueField {
    pub label: String,
    pub value: Option<ScalarValue>,
}

pub struct MeasurementField {
    pub label: String,
    pub value: Option<Option<f64>>,
}