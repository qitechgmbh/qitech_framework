use crossterm::event::KeyCode;
use qitech_framework_core::ScalarValue;
use qitech_framework_core::report::ConfigPropertyEvent;
use qitech_framework_core::report::WriteCapability;
use qitech_framework_core::schema::ConfigPropertyKind;
use ratatui::Frame;
use ratatui::layout::Constraint;
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
use ratatui::widgets::TableState;

use crate::types::AppAction;
use crate::types::ConfigFieldState;
use crate::widgets::machines_view::MachinesContext;
use crate::widgets::tab_view::TabItem;

#[derive(Default)]
pub enum Mode {
    #[default]
    Navigate,
    History(usize),
    Inspect(usize),
    Editing(Edit),
}

pub struct Edit {
    dirty: bool,
    value: String,
}

#[derive(Default)]
pub struct ConfigPage {
    selected: usize,
    mode: Mode,
}

impl TabItem<MachinesContext> for ConfigPage {
    fn on_key(&mut self, code: KeyCode, ctx: MachinesContext) -> Result<AppAction, KeyCode> {
        let machine = unsafe { &*ctx.selected };

        match self.mode {
            Mode::Navigate => self.onke,
            Mode::History(_) => todo!(),
            Mode::Inspect(_) => todo!(),
            Mode::Editing(edit) => todo!(),
        }

        if let Some(mut edit) = self.edit.take() {
            match code {
                KeyCode::Esc => {}
                KeyCode::Enter => {
                    let (key, field) = machine.config.get_index(self.selected).unwrap();

                    let value = match &field.kind {
                        ConfigPropertyKind::Enum { variants, .. } => {
                            if !variants.contains_name(&edit.value) {
                                return Ok(AppAction::NoAction);
                            }

                            ScalarValue::Enum(Some(edit.value))
                        }

                        ConfigPropertyKind::String => {
                            // TODO: capability check
                            ScalarValue::String(Some(edit.value))
                        }

                        ConfigPropertyKind::Boolean => {
                            let value = match edit.value.parse::<bool>() {
                                Ok(v) => v,
                                Err(_) => return Ok(AppAction::NoAction),
                            };

                            ScalarValue::Boolean(Some(value))
                        }

                        ConfigPropertyKind::Integer => {
                            let value = match edit.value.parse::<i64>() {
                                Ok(v) => v,
                                Err(_) => return Ok(AppAction::NoAction),
                            };

                            ScalarValue::Integer(Some(value))
                        }

                        ConfigPropertyKind::Float { .. } => {
                            let value = match edit.value.parse::<f64>() {
                                Ok(v) => v,
                                Err(_) => return Ok(AppAction::NoAction),
                            };

                            ScalarValue::Float(Some(value))
                        }
                    };

                    return Ok(AppAction::SetConfig {
                        machine: machine.ident,
                        resource: key.clone(),
                        value,
                    });
                }

                KeyCode::Char(c) => {
                    if !edit.dirty {
                        // first key replaces original value
                        edit.value.clear();
                        edit.dirty = true;
                    }

                    edit.value.push(c);
                    self.edit = Some(edit)
                }

                KeyCode::Backspace => {
                    edit.dirty = true;
                    edit.value.pop();
                    self.edit = Some(edit);
                }

                _ => {
                    self.edit = Some(edit);
                }
            }

            Ok(AppAction::NoAction)
        } else {
            match code {
                KeyCode::Up => {
                    if self.selected == 0 {
                        return Err(code);
                    }

                    self.selected = self.selected.saturating_sub(1);
                }

                KeyCode::Down => {
                    let max = machine.config.len().saturating_sub(1);
                    self.selected = (self.selected + 1).min(max);
                }

                KeyCode::Backspace => {
                    let (key, field) = machine.config.get_index(self.selected).expect("oops");

                    let ConfigFieldState::Initialized {
                        default,
                        capability: writeable,
                        ..
                    } = &field.state
                    else {
                        return Ok(AppAction::NoAction);
                    };

                    if writeable.forbidden() {
                        return Ok(AppAction::NoAction);
                    }

                    return Ok(AppAction::SetConfig {
                        machine: machine.ident,
                        resource: key.clone(),
                        value: default.clone(),
                    });
                }

                KeyCode::Enter => {
                    let (_, field) = machine.config.get_index(self.selected).expect("oops");

                    let ConfigFieldState::Initialized {
                        value,
                        capability: writeable,
                        ..
                    } = &field.state
                    else {
                        return Ok(AppAction::NoAction);
                    };

                    if writeable.forbidden() {
                        return Ok(AppAction::NoAction);
                    }

                    self.edit = Some(Edit {
                        dirty: false,
                        value: value.to_string(),
                    })
                }

                _ => return Err(code),
            }

            Ok(AppAction::NoAction)
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect, ctx: MachinesContext, in_focus: bool) {
        let machine = unsafe { &*ctx.selected };

        let rows: Vec<Row> = machine
            .config
            .iter()
            .enumerate()
            .map(|(i, (_, field))| {
                let selected = i == self.selected;
                let editing = selected && self.edit.is_some();

                let (display_value, writeable) = match &field.state {
                    ConfigFieldState::NotInitialized => (
                        "N/A".to_string(),
                        &WriteCapability::Forbidden {
                            reason: "Not initialized".to_string(),
                        },
                    ),

                    ConfigFieldState::Initialized {
                        value,
                        default: _,
                        capability: writeable,
                        constraints: _,
                    } => {
                        let display = format!("{value}");
                        (display, writeable)
                    }
                };

                let writable = writeable.is_allowed();

                let style = if editing {
                    Style::reset().fg(Color::Red)
                } else if selected && in_focus {
                    if writable {
                        Style::reset().fg(Color::LightBlue)
                    } else {
                        Style::reset()
                            .fg(Color::LightBlue)
                            .add_modifier(Modifier::DIM)
                    }
                } else if !writable {
                    Style::reset().fg(Color::Gray)
                } else {
                    Style::reset()
                };

                let value = if selected {
                    match &self.edit {
                        Some(edit) => edit.value.clone(),
                        None => display_value,
                    }
                } else {
                    display_value
                };

                Row::new(vec![Cell::from(field.label.clone()), Cell::from(value)]).style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [Constraint::Percentage(60), Constraint::Percentage(40)],
        )
        .style(Style::reset());

        frame.render_widget(table, area);
    }
}

// --- navigate ---
impl ConfigPage {
    fn on_key_navigate(
        &mut self,
        code: KeyCode,
        ctx: MachinesContext,
    ) -> Result<AppAction, KeyCode> {
        let machine = unsafe { &*ctx.selected };

        match code {
            KeyCode::Up => {
                if self.selected == 0 {
                    return Err(code);
                }

                self.selected -= 1;
            }

            KeyCode::Down => {
                let max = machine.state.len().saturating_sub(1);
                self.selected = (self.selected + 1).min(max);
            }

            KeyCode::Char(' ') => {
                self.mode = Mode::History(0);
            }

            _ => return Err(code),
        }

        Ok(AppAction::NoAction)
    }

    fn render_navigate(&self, frame: &mut Frame, area: Rect, ctx: MachinesContext, in_focus: bool) {
        let machine = unsafe { &*ctx.selected };

        let visible = area.height as usize;
        let total = machine.config.len();

        let offset = if total <= visible {
            0
        } else {
            self.selected
                .saturating_sub(visible / 2)
                .min(total - visible)
        };

        let rows: Vec<Row> = machine
            .config
            .iter()
            .skip(offset)
            .take(visible)
            .enumerate()
            .map(|(visible_index, (_, field))| {
                let index = offset + visible_index;

                let style = if index == self.selected && in_focus {
                    Style::default().fg(Color::LightBlue)
                } else {
                    Style::default()
                };

                let value = match &field.state {
                    ConfigFieldState::NotInitialized => "N/A".to_string(),
                    ConfigFieldState::Initialized { value, .. } => value.to_string(),
                };

                Row::new(vec![Cell::from(field.label.as_str()), Cell::from(value)]).style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [Constraint::Percentage(60), Constraint::Percentage(40)],
        )
        .style(Style::default());

        frame.render_widget(table, area);
    }
}

// --- history ---
impl ConfigPage {
    fn on_key_history(
        &mut self,
        code: KeyCode,
        ctx: MachinesContext,
        pos: usize,
    ) -> Result<AppAction, KeyCode> {
        let machine = unsafe { &*ctx.selected };

        let (_, field) = machine.state.get_index(self.selected).unwrap();

        match code {
            KeyCode::Esc => {
                self.mode = Mode::Navigate;
            }

            KeyCode::Char(' ') => {
                self.mode = Mode::Inspect(pos);
            }

            // don't consume exit button
            KeyCode::Char('q') => return Err(code),

            KeyCode::Up => {
                self.mode = Mode::History(pos.saturating_sub(1));
            }

            KeyCode::Down => {
                let max = field.records.len().saturating_sub(1);
                self.mode = Mode::History((pos + 1).min(max));
            }

            _ => {}
        }

        Ok(AppAction::NoAction)
    }

    fn render_history(&self, frame: &mut Frame, area: Rect, ctx: MachinesContext, pos: usize) {
        let machine = unsafe { &*ctx.selected };

        let (name, field) = machine
            .config
            .get_index(self.selected)
            .expect("Selected invalid item");

        let rows = field.records.iter().rev().map(|record| {
            let (event, value) = match &record.event {
                ConfigPropertyEvent::Registered {
                    default,
                    capability,
                    constraints,
                } => (
                    "Registered".to_string(),
                    format!(
                        "default={default:?}, writable={capability:?}, constraints={constraints:?}"
                    ),
                ),

                ConfigPropertyEvent::DefaultChanged { before, after } => (
                    "DefaultChanged".to_string(),
                    format!("{before:?} -> {after:?}"),
                ),

                ConfigPropertyEvent::CapabilityChanged { before, after } => (
                    "CapabilityChanged".to_string(),
                    format!("{before:?} -> {after:?}"),
                ),

                ConfigPropertyEvent::ConstraintsChanged { before, after } => (
                    "ConstraintsChanged".to_string(),
                    format!("{before:?} -> {after:?}"),
                ),

                ConfigPropertyEvent::Written {
                    value,
                    origin,
                    outcome,
                } => (
                    format!("Written ({origin:?})"),
                    format!("{value:?} => {outcome:?}"),
                ),
            };

            Row::new([
                record.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                event,
                value,
            ])
        });

        let table = Table::new(
            rows,
            [
                Constraint::Length(19),
                Constraint::Length(12),
                Constraint::Min(1),
            ],
        )
        .header(
            Row::new(["Timestamp", "Event", "Value"])
                .style(Style::default().add_modifier(Modifier::BOLD)),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Events ({name}) "))
                .border_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .column_spacing(4)
        .row_highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

        let mut state = TableState::default();
        state.select(Some(pos));

        frame.render_stateful_widget(table, area, &mut state);
    }
}

// --- inspect ---
impl ConfigPage {
    fn on_key_inspect(&mut self, code: KeyCode, pos: usize) -> Result<AppAction, KeyCode> {
        match code {
            KeyCode::Esc => {
                self.mode = Mode::History(pos);
                Ok(AppAction::NoAction)
            }

            // don't consume exit button
            KeyCode::Char('q') => Err(code),

            // consume other keys
            _ => Ok(AppAction::NoAction),
        }
    }

    fn render_inspect(&self, frame: &mut Frame, area: Rect, ctx: MachinesContext, pos: usize) {
        let machine = unsafe { &*ctx.selected };

        let (name, field) = machine
            .config
            .get_index(self.selected)
            .expect("Selected invalid item");

        let text = field
            .records
            .iter()
            .rev()
            .nth(pos)
            .map(|record| format!("{record:#?}"))
            .unwrap_or_else(|| "Invalid record".into());

        let paragraph = Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Inspect ({name}) "))
                .border_style(
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
        );

        frame.render_widget(paragraph, area);
    }
}

// --- edit ---
impl ConfigPage {
    fn on_key_edit(&mut self, code: KeyCode) -> Result<AppAction, KeyCode> {
        Err(code)
    }

    fn render_edit(&self, frame: &mut Frame, area: Rect, ctx: MachinesContext, pos: usize) {
        let machine = unsafe { &*ctx.selected };

        let (name, field) = machine
            .config
            .get_index(self.selected)
            .expect("Selected invalid item");

        let text = field
            .records
            .iter()
            .rev()
            .nth(pos)
            .map(|record| format!("{record:#?}"))
            .unwrap_or_else(|| "Invalid record".into());

        let paragraph = Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Inspect ({name}) "))
                .border_style(
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
        );

        frame.render_widget(paragraph, area);
    }
}
