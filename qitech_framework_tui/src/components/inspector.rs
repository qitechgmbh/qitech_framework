use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;

pub struct InspectMenu {
    scroll: u16,
    label: String,
    content: String,
}

impl InspectMenu {
    pub fn new(label: String, content: String) -> Self {
        Self {
            scroll: 0,
            label,
            content,
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn on_key(&mut self, code: KeyCode) -> Result<(), KeyCode> {
        let limit = self.content.lines().count().saturating_sub(1) as u16;

        match code {
            KeyCode::Up => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            KeyCode::Down => {
                self.scroll = self.scroll.saturating_add(1).min(limit);
            }
            _ => return Err(code),
        }

        Ok(())
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Inspect ({}) ", self.label))
            .border_style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            );

        let inner = block.inner(area);

        let content_lines = self.content.lines().count();

        // Like Navigation's `offset`: once we reach the end,
        // keep the last line at the bottom of the viewport.
        let max_scroll = content_lines.saturating_sub(inner.height as usize);

        let scroll = (self.scroll as usize).min(max_scroll) as u16;

        let paragraph = Paragraph::new(self.content.as_str())
            .scroll((scroll, 0))
            .block(block);

        frame.render_widget(paragraph, area);
    }
}
