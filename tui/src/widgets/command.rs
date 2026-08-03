use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Cell;
use ratatui::widgets::Row;
use ratatui::widgets::Table;

// space on state -> list of all mutations that happened
// space on config -> list of all mutations that happened
// space on measurements -> show chart
use crate::types::AppAction;
use crate::widgets::machines_view::MachinesContext;
use crate::widgets::tab_view::TabItem;

#[derive(Default)]
pub struct CommandsView {
    selected: usize,
}

impl TabItem<MachinesContext> for CommandsView {
    fn on_key(&mut self, code: KeyCode, ctx: MachinesContext) -> Result<AppAction, KeyCode> {
        let machine = unsafe { &*ctx.machine };

        match code {
            KeyCode::Up => {
                if self.selected == 0 {
                    return Err(code);
                }

                self.selected = self.selected.saturating_sub(1);
            }

            KeyCode::Down => {
                let max = machine.state.len().saturating_sub(1);
                self.selected = (self.selected + 1).min(max);
            }

            KeyCode::Enter => {
                let (key, _) = machine.commands.get_index(self.selected).unwrap();
                return Ok(AppAction::ExecuteCommand {
                    machine: machine.ident,
                    resource: key.clone(),
                });
            }

            _ => return Err(code),
        }

        Ok(AppAction::NoAction)
    }

    fn render(&self, frame: &mut Frame, area: Rect, ctx: MachinesContext, in_focus: bool) {
        let machine = unsafe { &*ctx.machine };

        // Number of rows that fit in the available area.
        // If you later wrap the table in a Block, subtract 2 for the borders.
        let visible = area.height as usize;

        let total = machine.state.len();

        let offset = if total <= visible {
            0
        } else {
            self.selected
                .saturating_sub(visible / 2)
                .min(total - visible)
        };

        let rows: Vec<Row> = machine
            .commands
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

                Row::new(vec![Cell::from(field.label.as_str())]).style(style)
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
