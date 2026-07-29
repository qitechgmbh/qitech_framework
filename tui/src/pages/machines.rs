use std::collections::HashMap;

use indexmap::IndexMap;
use qitech_framework::MachineIdentification;
use qitech_framework::MachineIdentificationUnique;
use qitech_framework::ScalarValue;
use qitech_framework_common::MachineSchema;
use qitech_framework_common::MachinesReport;
use qitech_framework_common::RuntimeRequestKind;
use qitech_framework_common::schema::ConfigPropertyValue;
use qitech_framework_common::schema::MeasurementValue;
use qitech_framework_common::schema::Node;
use qitech_framework_common::schema::NodeKind;
use qitech_framework_common::schema::StatePropertyValue;
use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Cell;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Row;
use ratatui::widgets::Table;
use ratatui::widgets::Tabs;

use crate::pages::Page;
use crate::styles;

#[derive(Debug, Clone)]
pub struct Cursor {
    pub machine: usize,
    pub section: usize,
    pub field: usize,
}

enum VerticalPosition {
    Outside,
    Tab,
    Config(usize),
    State(usize),
    Measurement(usize),
}

pub struct MachinesPage {
    schemas: HashMap<MachineIdentification, MachineSchema>,
    machines: Vec<MachineEntry>,

    cursor: Cursor,
    editing: Option<String>,
}

impl Page for MachinesPage {
    fn up(&mut self) -> bool {
        use VerticalPosition::*;

        self.pos_v = match self.pos_v {
            Outside => return true,

            Tab => {
                self.pos_v = Outside;
                return true;
            }

            Config(0) => Tab,

            State(0) => {
                let entry = self.selected_machine();

                if !entry.config.is_empty() {
                    Config(entry.config.len() - 1)
                } else {
                    Tab
                }
            }

            Measurement(0) => {
                let entry = self.selected_machine();

                if !entry.state.is_empty() {
                    State(entry.state.len() - 1)
                } else if !entry.config.is_empty() {
                    Config(entry.config.len() - 1)
                } else {
                    Tab
                }
            }

            Config(i) => Config(i - 1),
            State(i) => State(i - 1),
            Measurement(i) => Measurement(i - 1),
        };

        false
    }

    fn down(&mut self) {
        use VerticalPosition::*;
        let entry = &mut self.machines[self.pos_h];

        self.pos_v = match self.pos_v {
            Outside => Tab,

            Tab => {
                if entry.config.is_empty() {
                    if entry.state.is_empty() {
                        if entry.measurements.is_empty() {
                            Tab
                        } else {
                            Measurement(0)
                        }
                    } else {
                        State(0)
                    }
                } else {
                    Config(0)
                }
            }

            Config(i) if i + 1 < entry.config.len() => Config(i + 1),
            Config(i) => {
                if !entry.state.is_empty() {
                    State(0)
                } else if !entry.measurements.is_empty() {
                    Measurement(0)
                } else {
                    Config(i)
                }
            }

            State(i) if i + 1 < entry.state.len() => State(i + 1),

            State(i) => {
                if !entry.measurements.is_empty() {
                    Measurement(0)
                } else {
                    State(i)
                }
            }

            Measurement(i) if i + 1 < entry.measurements.len() => Measurement(i + 1),
            Measurement(i) => Measurement(i),
        };
    }

    fn can_edit(&self) -> bool {
        matches!(self.pos_v, VerticalPosition::Config(_))
    }

    fn edit_to_request(&mut self, value: String) -> RuntimeRequestKind {
        let VerticalPosition::Config(i) = self.pos_v else { unreachable!() };

        let machine =  self.selected_machine();
        let (resource, _) = machine.config.get_index(i).unwrap();

        RuntimeRequestKind::SetMachineConfiguration { 
            target: machine.ident,
            resource: resource.clone(), 
            value,
        }
    }

    fn display(&self, frame: &mut Frame, chunk: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Tabs
                Constraint::Min(0),    // Content
            ])
            .split(chunk);

        // --- draw tabs ---
        self.draw_tabs(frame, chunks[0]);

        // --- draw machines ---
        let chunk = chunks[1];

        let index = self.pos_h.min(self.machines.len());
        let Some(machine) = self.machines.get(index) else {
            // no machines present
            self.draw_no_machines(frame, chunk);
            return;
        };

        self.draw_machine(frame, chunk, machine);
    }
}

impl MachinesPage {
    fn selected_machine(&mut self) -> &mut MachineEntry {
        &mut self.machines[self.pos_h]
    }

    pub fn new(schemas: HashMap<MachineIdentification, MachineSchema>) -> Self {
        Self {
            schemas,
            machines: Default::default(),
            pos_h: 0,
            pos_v: VerticalPosition::Outside,
        }
    }

    pub fn add_machine(&mut self, ident_unique: MachineIdentificationUnique) {
        let ident = ident_unique.identification;

        let schema = self.schemas.get(&ident).unwrap();

        let mut config = IndexMap::new();
        collect_config_fields("", &schema.config_properties, &mut config);

        let mut state = IndexMap::new();
        collect_state_fields("", &schema.state_properties, &mut state);

        let mut measurements = IndexMap::new();
        collect_measurement_fields("", &schema.measurements, &mut measurements);

        self.machines.push(MachineEntry {
            title: schema.name.clone(),
            ident: ident_unique,
            config,
            state,
            measurements,
        });
    }

    #[allow(unused)]
    pub fn remove_machine(&mut self, ident: MachineIdentificationUnique) {
        self.machines.retain(|entry| entry.ident != ident);
    }

    /// --- refresh data ---
    pub fn handle_report(&mut self, report: MachinesReport) {
        for mutation in &report.config_mutations {
            let Some(entry) = self.find_machine(mutation.machine) else {
                continue;
            };

            let Some(item) = entry.config.get_mut(&mutation.path) else {
                continue;
            };

            item.value = Some(mutation.value.clone());
        }

        for mutation in &report.state_mutations {
            let Some(entry) = self.find_machine(mutation.machine) else {
                continue;
            };

            let Some(item) = entry.state.get_mut(&mutation.path) else {
                continue;
            };

            item.value = Some(mutation.value.clone());
        }

        for measurement in &report.measurements {
            let Some(entry) = self.find_machine(*measurement.machine) else {
                continue;
            };

            let Some(item) = entry.measurements.get_mut(measurement.path) else {
                continue;
            };

            item.value = Some(*measurement.value);
        }
    }

    fn draw_tabs(&self, frame: &mut Frame, chunk: Rect) {
        let style = match self.pos_v {
            VerticalPosition::Tab => styles::on_hover(),
            _ => Style::default(),
        };

        let mut titles = Vec::new();
        for page in &self.machines {
            titles.push(format!("{} ({})", page.title.clone(), page.ident.serial));
        }

        let tabs = Tabs::new(titles)
            .select(self.pos_h)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(style),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(tabs, chunk);
    }

    fn draw_no_machines(&self, frame: &mut Frame, chunk: Rect) {
        const PARAGRAPH: &str = "No Machines detected";

        let error = Paragraph::new(PARAGRAPH)
            .alignment(Alignment::Center)
            .block(Block::default());

        let area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .split(chunk)[1];

        frame.render_widget(error, area);
    }

    // ---
    fn draw_machine(&self, frame: &mut Frame, chunk: Rect, page: &MachineEntry) {
        let chunks = Layout::vertical([
            Constraint::Length(2 + page.config.len() as u16),
            Constraint::Length(2 + page.state.len() as u16),
            Constraint::Length(2 + page.measurements.len() as u16),
        ])
        .split(chunk);

        self.draw_config(frame, chunks[0], &page.config);
        self.draw_state(frame, chunks[1], &page.state);
        self.draw_measurement(frame, chunks[2], &page.measurements);
    }

    fn draw_config(&self, frame: &mut Frame, chunk: Rect, items: &IndexMap<String, ConfigField>) {
        const TITLE: &str = " Config ";

        let style = match self.pos_v {
            VerticalPosition::Config(_) => styles::on_hover(),
            _ => Style::default(),
        };

        let selected = match self.pos_v {
            VerticalPosition::Config(i) => Some(i),
            _ => None,
        };

        let rows: Vec<Row> = items
            .iter()
            .enumerate()
            .map(|(i, (_, field))| {
                let style = if selected == Some(i) {
                    Style::default().fg(Color::LightBlue)
                } else {
                    Style::default()
                };

                let value = match &field.value {
                    Some(v) => format!("{}", v.clone()),
                    None => "N/A".to_string(),
                };

                Row::new(vec![Cell::from(field.label.clone()), Cell::from(value)]).style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [Constraint::Percentage(60), Constraint::Percentage(40)],
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(style)
                .title(TITLE),
        );

        frame.render_widget(table, chunk);
    }

    fn draw_state(&self, frame: &mut Frame, chunk: Rect, items: &IndexMap<String, StateField>) {
        const TITLE: &str = " State ";

        let border_style = match self.pos_v {
            VerticalPosition::State(_) => styles::on_hover(),
            _ => Style::default(),
        };

        let selected = match self.pos_v {
            VerticalPosition::State(i) => Some(i),
            _ => None,
        };

        let rows: Vec<Row> = items
            .iter()
            .enumerate()
            .map(|(i, (_, field))| {
                let style = if selected == Some(i) {
                    Style::default().fg(Color::LightBlue)
                } else {
                    Style::default()
                };

                let value = field
                    .value
                    .as_ref()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "N/A".to_string());

                Row::new(vec![
                    Cell::from(field.label.clone()),
                    Cell::from(value),
                ])
                .style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [Constraint::Percentage(60), Constraint::Percentage(40)],
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(TITLE),
        );

        frame.render_widget(table, chunk);
    }

    fn draw_measurement(
        &self,
        frame: &mut Frame,
        chunk: Rect,
        items: &IndexMap<String, MeasurementField>,
    ) {
        const TITLE: &str = " Measurements ";

        let style = match self.pos_v {
            VerticalPosition::Measurement(_) => styles::on_hover(),
            _ => Style::default(),
        };

        let selected = match self.pos_v {
            VerticalPosition::Measurement(i) => Some(i),
            _ => None,
        };

        // --- measurements ---
        let rows: Vec<Row> = items
            .iter()
            .enumerate()
            .map(|(i, (_, field))| {
                let style = if selected == Some(i) {
                    Style::default().fg(Color::LightBlue)
                } else {
                    Style::default()
                };

                let value = match &field.value {
                    Some(v) => match v {
                        Some(v) => format!("{v:.3}"),
                        None => "null".to_string(),
                    },
                    None => "N/A".to_string(),
                };

                Row::new(vec![Cell::from(field.label.clone()), Cell::from(value)]).style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [Constraint::Percentage(60), Constraint::Percentage(40)],
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(style)
                .title(TITLE),
        );

        frame.render_widget(table, chunk);
    }

    // --- machines ---
    fn find_machine(&mut self, ident: MachineIdentificationUnique) -> Option<&mut MachineEntry> {
        self.machines.iter_mut().find(|m| m.ident == ident)
    }
}

// --- types ---
pub struct MachineEntry {
    pub title: String,
    pub ident: MachineIdentificationUnique,
    pub config: IndexMap<String, ConfigField>,
    pub state: IndexMap<String, StateField>,
    pub measurements: IndexMap<String, MeasurementField>,
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
