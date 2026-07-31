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

pub struct DropDown {
    state: State,
    title: &'static str,
    selected: usize,
}

impl DropDown {
    pub fn new(title: &'static str) -> Self {
        Self {
            title,
            state: State::Closed,
            selected: 0,
        }
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn on_key(&mut self, code: KeyCode, limit: usize) -> Result<(), KeyCode> {
        let pos = match self.state {
            State::Closed => {
                return if let KeyCode::Enter = code {
                    self.state = State::Open(self.selected);
                    Ok(())
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

            KeyCode::Enter => {
                self.selected = pos;
                self.state = State::Closed;
            }

            _ => return Err(code),
        }

        Ok(())
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
                    .title(self.title)
                    .borders(Borders::ALL)
                    .border_style(border_style),
            );

            frame.render_widget(list, area);
            return;
        }

        match self.state {
            State::Closed => {
                let selected = variants
                    .get(self.selected)
                    .map(|v| ListItem::new(format!("{} v", v)))
                    .unwrap_or_else(|| ListItem::new(""));

                let list = List::new(vec![selected]).block(
                    Block::default()
                        .title(self.title)
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
                        .title(self.title)
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
