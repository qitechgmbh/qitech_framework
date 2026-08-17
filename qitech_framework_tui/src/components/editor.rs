use std::mem;

use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Row;
use ratatui::widgets::Table;

pub enum EditorAction {
    NoAction,
    Abort,
    Yield(String),
}

pub struct EditMenu {
    label: String,
    value: String,
    dirty: bool,
}

impl EditMenu {
    pub fn new(label: String, value: String) -> Self {
        Self {
            label,
            value,
            dirty: false,
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn on_key(&mut self, code: KeyCode) -> Result<EditorAction, KeyCode> {
        match code {
            KeyCode::Esc => return Ok(EditorAction::Abort),

            KeyCode::Enter => {
                return Ok(match self.dirty {
                    true => EditorAction::Yield(mem::take(&mut self.value)),
                    false => EditorAction::Abort,
                });
            }

            KeyCode::Backspace => {
                self.dirty = true;
                self.value.pop();
            }

            KeyCode::Char(c) => {
                if !self.dirty {
                    // first key replaces original value
                    self.value.clear();
                    self.dirty = true;
                }

                self.value.push(c);
            }

            _ => {}
        }

        Ok(EditorAction::NoAction)
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, metadata: Vec<Row>) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Edit ({}) ", self.label))
            .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .border_style(Style::default().fg(Color::Red));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).split(inner);

        // --- editor ---
        let input = Paragraph::new(self.value.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red)),
            )
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

        frame.render_widget(input, chunks[0]);

        // --- metadata ---
        let table = Table::new(metadata, [Constraint::Length(15), Constraint::Min(1)])
            .style(Style::default().fg(Color::Red));

        frame.render_widget(table, chunks[1]);
    }
}
