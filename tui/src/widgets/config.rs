use std::mem;

use crossterm::event::KeyCode;
use qitech_framework_core::NumericValue;
use qitech_framework_core::ScalarValue;
use qitech_framework_core::report::ConfigPropertyEvent;
use qitech_framework_core::report::ConfigPropertyWriteOutcome;
use qitech_framework_core::report::Constraints;
use qitech_framework_core::report::WriteCapability;
use qitech_framework_core::schema::ConfigPropertyKind;
use ratatui::Frame;
use ratatui::layout::Constraint;
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
use ratatui::widgets::TableState;

use crate::types::AppAction;
use crate::types::ConfigFieldState;
use crate::types::MachineEntry;
use crate::widgets::machines_view::MachinesContext;
use crate::widgets::tab_view::TabItem;

#[derive(Default, Clone)]
pub enum Mode {
    #[default]
    Navigate,
    History(usize),
    Inspect(usize),
    Editing(Edit),
}

#[derive(Default, Clone)]
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
        match self.mode.clone() {
            Mode::Navigate => self.on_key_navigate(code, ctx),
            Mode::History(pos) => self.on_key_events(code, ctx, pos),
            Mode::Inspect(pos) => self.on_key_inspect(code, pos),
            Mode::Editing(edit) => self.on_key_edit(code, ctx, edit),
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect, ctx: MachinesContext, in_focus: bool) {
        match &self.mode {
            Mode::Navigate => self.render_navigate(frame, area, ctx, in_focus),
            Mode::History(pos) => self.render_events(frame, area, ctx, *pos),
            Mode::Inspect(pos) => self.render_inspect(frame, area, ctx, *pos),
            Mode::Editing(edit) => self.render_edit(frame, area, ctx, edit),
        }
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
                let max = machine.config.len().saturating_sub(1);
                self.selected = (self.selected + 1).min(max);
            }

            KeyCode::Char(' ') => {
                self.mode = Mode::History(0);
            }

            KeyCode::Enter => {
                let (_, field) = machine.config.get_index(self.selected).expect("oops");

                let ConfigFieldState::Initialized {
                    value, capability, ..
                } = &field.state
                else {
                    return Ok(AppAction::NoAction);
                };

                if capability.forbidden() {
                    return Ok(AppAction::NoAction);
                }

                self.mode = Mode::Editing(Edit {
                    dirty: false,
                    value: value.to_string(),
                });
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

// --- events ---
impl ConfigPage {
    fn on_key_events(
        &mut self,
        code: KeyCode,
        ctx: MachinesContext,
        pos: usize,
    ) -> Result<AppAction, KeyCode> {
        let machine = unsafe { &*ctx.selected };

        let (_, field) = machine.config.get_index(self.selected).unwrap();

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

    fn render_events(&self, frame: &mut Frame, area: Rect, ctx: MachinesContext, pos: usize) {
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
                    format!("default={default}, write={capability:?}, constraints={constraints:?}"),
                ),

                ConfigPropertyEvent::DefaultChanged { before, after } => {
                    ("DefaultChanged".to_string(), format!("{before} -> {after}"))
                }

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
                } => {
                    let outcome = match outcome {
                        ConfigPropertyWriteOutcome::Accepted { .. } => "Accepted",
                        ConfigPropertyWriteOutcome::Rejected(_) => "Rejected",
                    };

                    (
                        format!("Written ({origin})"),
                        format!("{value} => {outcome}"),
                    )
                }
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
                Constraint::Length(32),
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
    fn on_key_edit(
        &mut self,
        code: KeyCode,
        ctx: MachinesContext,
        mut edit: Edit,
    ) -> Result<AppAction, KeyCode> {
        let machine = unsafe { &*ctx.selected };

        match code {
            KeyCode::Esc => self.mode = Mode::Navigate,
            KeyCode::Enter => {
                self.mode = Mode::Navigate;
                return Ok(self.submit_edit(edit, machine));
            }
            KeyCode::Char(c) => {
                if !edit.dirty {
                    // first key replaces original value
                    edit.value.clear();
                    edit.dirty = true;
                }

                edit.value.push(c);
                self.mode = Mode::Editing(edit);
            }

            KeyCode::Backspace => {
                edit.dirty = true;
                edit.value.pop();
                self.mode = Mode::Editing(edit);
            }

            _ => {
                self.mode = Mode::Editing(edit);
            }
        }

        Ok(AppAction::NoAction)
    }

    fn render_edit(&self, frame: &mut Frame, area: Rect, ctx: MachinesContext, edit: &Edit) {
        let machine = unsafe { &*ctx.selected };

        let (name, field) = machine
            .config
            .get_index(self.selected)
            .expect("Selected invalid item");

        let ConfigFieldState::Initialized { constraints, .. } = &field.state else {
            return;
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Edit ({name}) "))
            .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .border_style(Style::default().fg(Color::Red));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).split(inner);

        // --- editor ---
        let input = Paragraph::new(edit.value.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red)),
            )
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

        frame.render_widget(input, chunks[0]);

        // --- constraints ---
        let rows: Vec<Row> = match constraints {
            Constraints::None => vec![Row::default()],

            Constraints::Numeric { min, max, nullable } => {
                let mut rows = vec![Row::new(["Nullable".to_string(), nullable.to_string()])];

                rows.push(Row::new([
                    "Min".to_string(),
                    match min {
                        NumericValue::Integer(value) => match value {
                            Some(value) => format!("{value}"),
                            None => "null".to_string(),
                        },

                        NumericValue::Float(value) => match value {
                            Some(value) => format!("{value}"),
                            None => "null".to_string(),
                        },
                    },
                ]));

                rows.push(Row::new([
                    "Max".to_string(),
                    match max {
                        NumericValue::Integer(value) => match value {
                            Some(value) => format!("{value}"),
                            None => "null".to_string(),
                        },

                        NumericValue::Float(value) => match value {
                            Some(value) => format!("{value}"),
                            None => "null".to_string(),
                        },
                    },
                ]));

                rows
            }

            Constraints::String {
                min_length,
                max_length,
                pattern,
                nullable,
            } => {
                let mut rows = vec![Row::new(["Nullable".to_string(), nullable.to_string()])];

                if let Some(min_length) = min_length {
                    rows.push(Row::new(["Min length".to_string(), min_length.to_string()]));
                }

                if let Some(max_length) = max_length {
                    rows.push(Row::new(["Max length".to_string(), max_length.to_string()]));
                }

                if let Some(pattern) = pattern {
                    rows.push(Row::new(["Pattern".to_string(), pattern.to_string()]));
                }

                rows
            }

            Constraints::Enum { allowed, nullable } => vec![
                Row::new(["Allowed".to_string(), format!("{allowed:?}")]),
                Row::new(["Nullable".to_string(), nullable.to_string()]),
            ],
        };

        let table = Table::new(rows, [Constraint::Length(15), Constraint::Min(1)])
            .style(Style::default().fg(Color::Red));

        frame.render_widget(table, chunks[1]);
    }

    // -- util ---
    fn submit_edit(&self, edit: Edit, machine: &MachineEntry) -> AppAction {
        if !edit.dirty {
            return AppAction::NoAction;
        }

        let (key, field) = machine.config.get_index(self.selected).unwrap();

        let value = match &field.kind {
            ConfigPropertyKind::Enum { variants, .. } => {
                if !variants.contains_name(&edit.value) {
                    return AppAction::NoAction;
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
                    Err(_) => return AppAction::NoAction,
                };

                ScalarValue::Boolean(Some(value))
            }

            ConfigPropertyKind::Integer => {
                let value = match edit.value.parse::<i64>() {
                    Ok(v) => v,
                    Err(_) => return AppAction::NoAction,
                };

                ScalarValue::Integer(Some(value))
            }

            ConfigPropertyKind::Float { .. } => {
                let value = match edit.value.parse::<f64>() {
                    Ok(v) => v,
                    Err(_) => return AppAction::NoAction,
                };

                ScalarValue::Float(Some(value))
            }
        };

        AppAction::SetConfig {
            machine: machine.ident,
            resource: key.clone(),
            value,
        }
    }
}
