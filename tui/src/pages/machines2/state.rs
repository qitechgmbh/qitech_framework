use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Cell;
use ratatui::widgets::Row;
use ratatui::widgets::Table;

use crate::pages::TabAction;
use crate::pages::TabWidget;
use crate::pages::machines2::Context;

#[derive(Default)]
pub struct StatePage {
    selected: usize,
}

impl TabWidget<Context> for StatePage {
    fn can_enter(&self) -> bool {
        true
    }

    fn on_key(&mut self, code: KeyCode, ctx: Context) -> TabAction {
        let machine = unsafe { &*ctx.machine };

        match code {
            KeyCode::Up => {
                if self.selected == 0 {
                    return TabAction::Exit;
                }

                self.selected = self.selected.saturating_sub(1);
            }

            KeyCode::Down => {
                let max = machine.state.len().saturating_sub(1);
                self.selected = (self.selected + 1).min(max);
            }

            _ => {}
        }

        TabAction::no_action()
    }

    fn render(&self, frame: &mut Frame, area: Rect, ctx: Context, in_focus: bool) {
        const TITLE: &str = " State ";

        let machine = unsafe { &*ctx.machine };

        let border_style = if in_focus {
            Style::reset().fg(Color::LightBlue)
        } else {
            Style::reset()
        };

        let rows: Vec<Row> = machine
            .state
            .iter()
            .enumerate()
            .map(|(i, (_, field))| {
                let selected = i == self.selected;

                let style = if selected && in_focus {
                    Style::reset().fg(Color::LightBlue)
                } else {
                    Style::reset()
                };

                let value = match &field.value {
                    Some(v) => format!("{v}"),
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
                .border_style(border_style)
                .title(TITLE),
        );

        frame.render_widget(table, area);
    }
}
