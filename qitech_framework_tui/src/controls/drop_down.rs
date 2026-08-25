use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ratatui::widgets::ListState;

use crate::types::KeyResult;

#[derive(Clone, Copy)]
enum State {
    Closed,
    Open(usize),
}

pub struct DropDown {
    state: State,
    label: String,
    selected: usize,
}

impl DropDown {
    pub fn new(label: String) -> Self {
        Self {
            state: State::Closed,
            label,
            selected: 0,
        }
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn on_key(&mut self, code: KeyCode) -> KeyResult<()> {
        let pos = match self.state {
            State::Closed => {
                return if let KeyCode::Enter = code {
                    self.state = State::Open(self.selected);
                    KeyResult::Handled(())
                } else {
                    KeyResult::Bubble(code)
                };
            }
            State::Open(v) => v,
        };

        match code {
            KeyCode::Up => {
                self.state = State::Open(pos.saturating_sub(1));
            }

            KeyCode::Down => {
                self.state = State::Open(pos + 1);
            }

            // submitted
            KeyCode::Enter => {
                self.selected = pos;
                self.state = State::Closed;
            }

            // canceled
            KeyCode::Esc => {
                self.state = State::Closed;
            }

            _ => return KeyResult::Bubble(code),
        }

        KeyResult::Handled(())
    }

    pub fn rendered_height(&self, options: &[String]) -> usize {
        match self.state {
            State::Closed => 3,
            State::Open(_) => 2 + options.len(),
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, in_focus: bool, options: Vec<String>) {
        let border_style = if in_focus {
            Style::reset().fg(Color::Blue)
        } else {
            Style::reset().fg(Color::White)
        };

        if options.is_empty() {
            let list = List::new(vec![ListItem::new(Line::from(Span::styled(
                "<no items>",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )))])
            .block(
                Block::default()
                    .title(self.label.as_str())
                    .borders(Borders::ALL)
                    .border_style(border_style),
            );

            frame.render_widget(list, area);
            return;
        }

        match self.state {
            State::Closed => {
                let selected = options
                    .get(self.selected)
                    .map(|v| ListItem::new(format!("{} v", v)))
                    .unwrap_or_else(|| ListItem::new(""));

                let list = List::new(vec![selected]).block(
                    Block::default()
                        .title(self.label.as_str())
                        .borders(Borders::ALL)
                        .border_style(border_style),
                );

                frame.render_widget(list, area);
            }

            State::Open(selection) => {
                let items: Vec<ListItem> = options
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        let postfix = if i == selection { " <" } else { "  " };

                        ListItem::new(format!("{}{}", item, postfix)).style(if i == selection {
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                        })
                    })
                    .collect();

                let list = List::new(items).block(
                    Block::default()
                        .title(self.label.as_str())
                        .borders(Borders::ALL)
                        .border_style(border_style),
                );

                let mut state = ListState::default();
                state.select(Some(selection));

                frame.render_stateful_widget(list, area, &mut state);
            }
        }
    }
}
