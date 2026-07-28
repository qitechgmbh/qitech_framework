use std::io;

use crossterm::event::DisableMouseCapture;
use crossterm::event::EnableMouseCapture;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::MouseEvent;
use crossterm::event::MouseEventKind;
use crossterm::event::{self};
use crossterm::execute;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use indexmap::IndexMap;
use qitech_framework::ScalarValue;
use qitech_framework_common::MachineSchema;
use qitech_framework_common::schema::ConfigPropertyValue;
use qitech_framework_common::schema::MeasurementValue;
use qitech_framework_common::schema::Node;
use qitech_framework_common::schema::NodeKind;
use qitech_framework_common::schema::StatePropertyValue;
use ratatui::prelude::*;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Cell;
use ratatui::widgets::Row;
use ratatui::widgets::Table;

fn main() -> io::Result<()> {
    let schema = include_str!("laser_v1.yaml");
    let schema = MachineSchema::from_yaml_str(schema).unwrap();

    let mut config = Vec::new();
    collect_config_fields("", &schema.config_properties, &mut config);

    let mut state = Vec::new();
    collect_state_fields("", &schema.state_properties, &mut state);

    let mut measurements = Vec::new();
    collect_measurement_fields("", &schema.measurements, &mut measurements);

    let ui_model = UiModel {
        config,
        state,
        measurements,
    };

    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(schema, ui_model);

    while app.running {
        terminal.draw(|frame| ui(frame, &app))?;

        match event::read()? {
            Event::Key(key) => {
                app.handle_key(key.code);
            }

            Event::Mouse(mouse) => {
                app.handle_mouse(mouse);
            }

            _ => {}
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

struct App {
    schema: MachineSchema,
    model: UiModel,
    pointer: usize,
    pointer_enabled: bool,
    limit: usize,
    running: bool,
}

impl App {
    fn new(schema: MachineSchema, model: UiModel) -> Self {
        Self {
            schema,
            model,
            pointer: 0,
            pointer_enabled: false,
            limit: 0,
            running: true,
        }
    }

    fn handle_key(&mut self, key: KeyCode) {
        self.limit = self.model.config.len().saturating_sub(1);

        match key {
            KeyCode::Char('q') => self.running = false,
            KeyCode::Esc => self.pointer_enabled = false,
            KeyCode::Up => {
                self.pointer = self.pointer.saturating_sub(1);
                self.pointer_enabled = true;
            }
            KeyCode::Down => {
                self.pointer = (self.pointer + 1).min(self.limit);
                self.pointer_enabled = true;
            }
            _ => {}
        }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        if let MouseEventKind::Down(_) = mouse.kind {
            let row = mouse.row as usize;

            // select config property
            if row > 1 {
                let index = row - 2;

                if index < self.model.config.len() {
                    self.pointer = index;
                    self.pointer_enabled = true;
                }
            }
        }
    }
}

fn ui(frame: &mut Frame, app: &App) {
    let layout = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(frame.area());

    let outer = Block::default()
        .borders(Borders::ALL)
        .title(app.schema.name.as_str());

    let inner = outer.inner(frame.area());
    frame.render_widget(outer, frame.area());

    let chunks = Layout::vertical([
        Constraint::Length(2 + app.model.config.len() as u16),
        Constraint::Length(2 + app.model.state.len() as u16),
        Constraint::Length(2 + app.model.measurements.len() as u16),
    ])
    .split(inner);

    // --- config fields ---
    let rows: Vec<Row> = app
        .model
        .config
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let style = if app.pointer_enabled && i == app.pointer {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };

            Row::new(vec![Cell::from(field.label.clone()), Cell::from("N/A")]).style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [Constraint::Percentage(60), Constraint::Percentage(40)],
    )
    .block(Block::default().borders(Borders::ALL).title("Config"));

    frame.render_widget(table, chunks[0]);

    // --- state ---
    let rows: Vec<Row> = app
        .model
        .state
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let style = Style::default();

            Row::new(vec![Cell::from(field.label.clone()), Cell::from("N/A")]).style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [Constraint::Percentage(60), Constraint::Percentage(40)],
    )
    .block(Block::default().borders(Borders::ALL).title("State"));

    frame.render_widget(table, chunks[1]);

    // --- measurements ---
    let rows: Vec<Row> = app
        .model
        .measurements
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let style = Style::default();

            Row::new(vec![Cell::from(field.label.clone()), Cell::from("N/A")]).style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [Constraint::Percentage(60), Constraint::Percentage(40)],
    )
    .block(Block::default().borders(Borders::ALL).title("Measurements"));

    frame.render_widget(table, chunks[2]);
}

// --- types ---
struct UiModel {
    config: Vec<ConfigField>,
    state: Vec<StateField>,
    measurements: Vec<MeasurementField>,
}

struct ConfigField {
    label: String,
    value: ScalarValue,
}

struct StateField {
    label: String,
    value: ScalarValue,
}

struct MeasurementField {
    label: String,
    value: Option<f64>,
}

// --- utils ---
fn collect_config_fields(
    prefix: &str,
    properties: &IndexMap<String, Node<ConfigPropertyValue>>,
    fields: &mut Vec<ConfigField>,
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
                fields.push(ConfigField {
                    label: path.clone(),
                    value: ScalarValue::Float(Some(1.0)),
                });
            }
        }
    }
}

fn collect_state_fields(
    prefix: &str,
    properties: &IndexMap<String, Node<StatePropertyValue>>,
    fields: &mut Vec<StateField>,
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
                fields.push(StateField {
                    label: path.clone(),
                    value: ScalarValue::Float(Some(1.0)),
                });
            }
        }
    }
}

fn collect_measurement_fields(
    prefix: &str,
    properties: &IndexMap<String, Node<MeasurementValue>>,
    fields: &mut Vec<MeasurementField>,
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
                fields.push(MeasurementField {
                    label: path.clone(),
                    value: Some(1.0),
                });
            }
        }
    }
}
