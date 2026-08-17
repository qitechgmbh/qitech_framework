use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;

use super::Mode;
use super::StateView;
use crate::types::AppAction;
use crate::widgets::machines_view::MachinesContext;

impl StateView {
    pub fn on_key_inspect(&mut self, code: KeyCode, pos: usize) -> Result<AppAction, KeyCode> {
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

    pub fn render_inspect(&self, frame: &mut Frame, area: Rect, ctx: MachinesContext, pos: usize) {
        let machine = unsafe { &*ctx.selected };

        let (name, field) = machine
            .state
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
