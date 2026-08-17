use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Cell;
use ratatui::widgets::Row;
use ratatui::widgets::Table;

use super::Mode;
use super::StateView;
use crate::types::AppAction;
use crate::types::StatePropertyFieldState;
use crate::widgets::machines_view::MachinesContext;

impl StateView {
    pub fn on_key_navigate(
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

    pub fn render_navigate(
        &self,
        frame: &mut Frame,
        area: Rect,
        ctx: MachinesContext,
        in_focus: bool,
    ) {
        let machine = unsafe { &*ctx.selected };

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
            .state
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
                    StatePropertyFieldState::NotInitialized => "N/A".to_string(),
                    StatePropertyFieldState::Initialized { value } => value.to_string(),
                };

                Row::new(vec![Cell::from(field.label.as_str()), Cell::from(value)]).style(style)
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
