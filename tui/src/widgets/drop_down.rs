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

pub struct DropDown {
    title: String,
    selected: usize,
    selection: Option<usize>,
}

impl DropDown {
    pub fn new(title: String) -> Self {
        Self { title, selected: 0, selection: None }
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn select(&self) -> usize {
        self.selected
    }

    fn on_key(&mut self, code: KeyCode, limit: usize) -> Result<(), KeyCode> {
        match code {
            KeyCode::Up => {
                if let Some(v) = self.selection {
                    self.selection = Some((v + 1).min(limit));
                    Ok(())
                } else {
                    Err(code)
                }
            }

            KeyCode::Down => {
                if let Some(v) = self.selection {
                    self.selection = Some(v.saturating_sub(1));
                    Ok(())
                } else {
                    Err(code)
                }
            }

            KeyCode::Enter => {
                match self.selection {
                    Some(v) => self.selected = v,
                    None => self.selection = Some(self.selected),
                }

                Ok(())
            }

            _ => Err(code),
        }
    }

    fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        in_focus: bool,
        variants: Vec<ListItem>,
    ) {
        let border_style = if in_focus {
            Style::default().fg(Color::Blue)
        } else {
            Style::default().fg(Color::White)
        };

        if let Some(selection) = self.selection {
            let items: Vec<ListItem> = variants
                .into_iter()
                .enumerate()
                .map(|(i, item)| {
                    if i == selection {
                        item.style(
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else {
                        item
                    }
                })
                .collect();

            let list = List::new(items).block(
                Block::default()
                    .title(self.title.clone())
                    .borders(Borders::ALL)
                    .border_style(border_style),
            );

            let mut state = ListState::default();
            state.select(Some(selection));

            frame.render_stateful_widget(list, area, &mut state);
        }
    }
}
