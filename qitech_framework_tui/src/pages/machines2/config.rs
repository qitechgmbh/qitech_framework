use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Cell;
use ratatui::widgets::Row;
use ratatui::widgets::Table;

use crate::pages::TabAction;
use crate::pages::TabWidget;
use crate::pages::machines2::Context;
use crate::types::AppAction;

pub struct Edit {
    dirty: bool,
    value: String,
}

#[derive(Default)]
pub struct ConfigPage {
    selected: usize,
    edit: Option<Edit>,
}

impl TabWidget<Context> for ConfigPage {
    fn can_enter(&self) -> bool {
        true
    }

    fn on_key(&mut self, code: KeyCode, ctx: Context) -> TabAction {
        let machine = unsafe { &*ctx.machine };

        if let Some(mut edit) = self.edit.take() {
            match code {
                KeyCode::Esc => {}
                KeyCode::Enter => {
                    let (key, _) = machine.config.get_index(self.selected).unwrap();
                    return TabAction::AppAction(AppAction::SetConfig {
                        machine: machine.ident,
                        resource: key.clone(),
                        value: edit.value,
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
        } else {
            match code {
                KeyCode::Up => {
                    if self.selected == 0 {
                        return TabAction::Exit;
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
                        return TabAction::no_action();
                    };

                    self.edit = Some(Edit {
                        dirty: false,
                        value: value.to_string(),
                    })
                }

                _ => {}
            }
        }

        TabAction::no_action()
    }

    fn render(&self, frame: &mut Frame, area: Rect, ctx: Context, in_focus: bool) {
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
                    Style::reset().fg(Color::LightBlue)
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
