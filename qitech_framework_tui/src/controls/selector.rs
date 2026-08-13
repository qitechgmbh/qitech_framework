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

#[derive(Clone, Copy)]
enum State {
    Closed,
    Open(usize),
}

pub struct Picker {
    state: State,
    label: &'static str,
}

impl Picker {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            state: State::Closed,
        }
    }

    pub fn on_key(&mut self, code: KeyCode, limit: usize) -> Result<Option<usize>, KeyCode> {
        let pos = match self.state {
            State::Closed => {
                return if let KeyCode::Enter = code {
                    self.state = State::Open(0);
                    Ok(None)
                } else {
                    Err(code)
                };
            }
            State::Open(v) => v,
        };

        match code {
            KeyCode::Up => {
                self.state = State::Open(pos.saturating_sub(1));
            }

            KeyCode::Down => {
                self.state = State::Open((pos + 1).min(limit));
            }

            // submitted
            KeyCode::Enter => {
                self.state = State::Closed;
                return Ok(Some(pos));
            }

            // canceled
            KeyCode::Esc => {
                self.state = State::Closed;
            }

            _ => return Err(code),
        }

        Ok(None)
    }

    pub fn rendered_height<I, S>(&self, variants: I) -> usize
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        match self.state {
            State::Closed => 3,
            State::Open(_) => 2 + variants.into_iter().count(),
        }
    }

    pub fn render<I, S>(&self, frame: &mut Frame, area: Rect, in_focus: bool, variants: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let variants: Vec<String> = variants
            .into_iter()
            .map(|v| v.as_ref().to_string())
            .collect();

        let border_style = if in_focus {
            Style::reset().fg(Color::Blue)
        } else {
            Style::reset().fg(Color::White)
        };

        if variants.is_empty() {
            let list = List::new(vec![ListItem::new(Line::from(Span::styled(
                "<no items>",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )))])
            .block(
                Block::default()
                    .title(self.label)
                    .borders(Borders::ALL)
                    .border_style(border_style),
            );

            frame.render_widget(list, area);
            return;
        }

        match self.state {
            State::Closed => {
                let list = List::new(vec![ListItem::new(self.label).style(border_style)]).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(border_style),
                );

                frame.render_widget(list, area);
            }

            State::Open(selection) => {
                let items: Vec<ListItem> = variants
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        let prefix = if i == selection { "> " } else { "  " };

                        ListItem::new(format!("{}{}", prefix, item)).style(if i == selection {
                            Style::default()
                                .fg(Color::LightBlue)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                        })
                    })
                    .collect();

                let list = List::new(items).block(
                    Block::default()
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
