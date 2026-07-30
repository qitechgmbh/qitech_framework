use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
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
                self.state = State::Open((pos + 1).min(limit));
            }

            KeyCode::Down => {
                self.state = State::Open(pos.saturating_sub(1));
            }

            KeyCode::Enter => {
                self.selected = pos;
                self.state = State::Closed;
            }

            _ => return Err(code),
        }

        Ok(())
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, in_focus: bool, variants: &[ListItem<'_>]) {
        let border_style = if in_focus {
            Style::default().fg(Color::Blue)
        } else {
            Style::default().fg(Color::White)
        };

        if let State::Open(selection) = self.state {
            let items: Vec<ListItem> = variants
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    if i == selection {
                        item.clone().style(
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else {
                        item.clone()
                    }
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
