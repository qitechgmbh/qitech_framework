use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Cell;
use ratatui::widgets::Row;
use ratatui::widgets::Table;

#[derive(Default)]
pub struct Navigation {
    selected: usize,
}

impl Navigation {
    pub fn pos(&self) -> usize {
        self.selected
    }

    pub fn clamp(&mut self, limit: usize) {
        self.selected = self.selected.min(limit);
    }

    pub fn on_key(&mut self, code: KeyCode, limit: usize) -> Result<(), KeyCode> {
        match code {
            KeyCode::Up => {
                if self.selected == 0 {
                    return Err(code);
                }

                self.selected -= 1;
            }

            KeyCode::Down => {
                self.selected = (self.selected + 1).min(limit);
            }

            _ => return Err(code),
        }

        Ok(())
    }

    pub fn render<I>(&self, frame: &mut Frame, area: Rect, items: I, in_focus: bool)
    where
        I: IntoIterator<Item = (String, String)>,
    {
        let visible = area.height as usize;

        if visible == 0 {
            return;
        }

        let items: Vec<(String, String)> = items.into_iter().collect();
        let total = items.len();

        if total == 0 {
            return;
        }

        let selected = self.selected.min(total - 1);

        let offset = if total <= visible {
            0
        } else {
            selected.saturating_sub(visible / 2).min(total - visible)
        };

        let rows: Vec<Row> = items
            .iter()
            .skip(offset)
            .take(visible)
            .enumerate()
            .map(|(visible_index, (label, value))| {
                let index = offset + visible_index;

                let style = if index == selected && in_focus {
                    Style::reset().fg(Color::LightBlue)
                } else {
                    Style::reset()
                };

                Row::new(vec![Cell::from(label.as_str()), Cell::from(value.as_str())]).style(style)
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
