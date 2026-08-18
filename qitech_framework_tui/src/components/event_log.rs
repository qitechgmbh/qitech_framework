use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Row;
use ratatui::widgets::Table;
use ratatui::widgets::TableState;

pub enum EventLogViewAction {
    NoAction,
    Inspect(usize),
    Bubble(KeyCode),
    Exit,
}

pub struct EventLogContent<'a, I>
where
    I: Iterator<Item = Row<'a>>,
{
    pub rows: I,
    pub cols: Vec<(Constraint, String)>,
}

#[derive(Clone)]
pub struct EventLogMenu {
    pos: usize,
    label: String,
}

impl EventLogMenu {
    pub fn new(label: String) -> Self {
        Self { pos: 0, label }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn on_key(&mut self, code: KeyCode, limit: usize) -> EventLogViewAction {
        match code {
            KeyCode::Esc => return EventLogViewAction::Exit,
            KeyCode::Up => self.pos = self.pos.saturating_sub(1),
            KeyCode::Down => self.pos = self.pos.saturating_add(1).min(limit),

            // Consume navigation buttons
            KeyCode::Left | KeyCode::Right => return EventLogViewAction::NoAction,

            KeyCode::Char(' ') => return EventLogViewAction::Inspect(self.pos),
            _ => return EventLogViewAction::Bubble(code),
        }

        EventLogViewAction::NoAction
    }

    pub fn render<'a, I: Iterator<Item = Row<'a>>>(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        content: EventLogContent<'a, I>,
    ) {
        let headers = content.cols.iter().map(|(_, name)| name.as_str());
        let widths = content.cols.iter().map(|(constraint, _)| *constraint);

        let table = Table::new(content.rows, widths)
            .header(Row::new(headers).style(Style::reset().add_modifier(Modifier::BOLD)))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Events ({}) ", self.label))
                    .border_style(
                        Style::reset()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .column_spacing(4)
            .row_highlight_style(
                Style::reset()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        let mut state = TableState::default();
        state.select(Some(self.pos));

        frame.render_stateful_widget(table, area, &mut state);
    }
}
