use crossterm::event::KeyCode;
use qitech_framework_core::report::StatePropertyEvent;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Row;
use ratatui::widgets::Table;
use ratatui::widgets::TableState;

use super::Mode;
use super::StateView;
use crate::types::AppAction;
use crate::widgets::machines_view::MachinesContext;

impl StateView {
    pub fn on_key_history(
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

    pub fn render_history(&self, frame: &mut Frame, area: Rect, ctx: MachinesContext, pos: usize) {
        let machine = unsafe { &*ctx.selected };

        let (name, field) = machine
            .state
            .get_index(self.selected)
            .expect("Selected invalid item");

        let rows = field.records.iter().rev().map(|record| {
            let (event, value) = match &record.event {
                StatePropertyEvent::Registered { value } => ("Registered", format!("{value}")),
                StatePropertyEvent::ValueChanged { value } => ("Value Changed", format!("{value}")),
            };

            Row::new([
                record.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                event.to_string(),
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
