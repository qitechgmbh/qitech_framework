use crossterm::event::KeyCode;
use qitech_framework_core::ScalarValue;
use qitech_framework_core::schema::ConfigPropertyKind;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::widgets::Cell;
use ratatui::widgets::Row;
use ratatui::widgets::Table;

use crate::types::AppAction;
use crate::widgets::machines_view::MachinesContext;
use crate::widgets::tab_view::TabItem;

pub struct Edit {
    dirty: bool,
    value: String,
}

#[derive(Default)]
pub struct ConfigPage {
    selected: usize,
    edit: Option<Edit>,
}

impl TabItem<MachinesContext> for ConfigPage {
    fn on_key(&mut self, code: KeyCode, ctx: MachinesContext) -> Result<AppAction, KeyCode> {
        let machine = unsafe { &*ctx.machine };

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

                        ConfigPropertyKind::String { .. } => {
                            // TODO: capability check
                            ScalarValue::String(Some(edit.value))
                        }

                        ConfigPropertyKind::Boolean { .. } => {
                            let value = match edit.value.parse::<bool>() {
                                Ok(v) => v,
                                Err(_) => return Ok(AppAction::NoAction),
                            };

                            ScalarValue::Boolean(Some(value))
                        }

                        ConfigPropertyKind::Integer { .. } => {
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

                KeyCode::Enter => {
                    let (_, field) = machine.config.get_index(self.selected).expect("oops");

                    // if value is N/A we can't set it
                    let Some(value) = &field.value else {
                        return Ok(AppAction::NoAction);
                    };

                    if !field.can_write {
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
        let machine = unsafe { &*ctx.machine };

        let rows: Vec<Row> = machine
            .config
            .iter()
            .enumerate()
            .map(|(i, (_, field))| {
                let selected = i == self.selected;
                let editing = selected && self.edit.is_some();

                let style = if editing {
                    Style::reset().fg(Color::Red)
                } else if selected && in_focus {
                    if field.can_write {
                        Style::reset().fg(Color::LightBlue)
                    } else {
                        Style::reset()
                            .fg(Color::LightBlue)
                            .add_modifier(Modifier::DIM)
                    }
                } else if !field.can_write {
                    Style::reset().fg(Color::Gray)
                } else {
                    Style::reset()
                };

                let default = match &field.value {
                    Some(v) => format!("{v}"),
                    None => "N/A".to_string(),
                };

                let value = if selected {
                    match &self.edit {
                        Some(edit) => edit.value.clone(),
                        None => default,
                    }
                } else {
                    default
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
