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

pub enum HistoryAction {
    NoAction,
    Inspect(usize),
    Bubble(KeyCode),
    Exit,
}

pub struct HistoryContent<'a, I>
where
    I: Iterator<Item = Row<'a>>,
{
    pub rows: I,
    pub columns: Vec<(Constraint, String)>,
}

pub struct HistoryMenu {
    pos: usize,
    label: String,
}

impl HistoryMenu {
    pub fn new(label: String) -> Self {
        Self { pos: 0, label }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn on_key(&mut self, code: KeyCode, limit: usize) -> HistoryAction {
        match code {
            KeyCode::Esc => HistoryAction::Exit,

            KeyCode::Up => {
                self.pos = self.pos.saturating_sub(1);
                HistoryAction::NoAction
            }

            KeyCode::Down => {
                self.pos = (self.pos + 1).min(limit);
                HistoryAction::NoAction
            }

            KeyCode::Char(' ') => HistoryAction::Inspect(self.pos),

            _ => HistoryAction::Bubble(code),
        }
    }

    pub fn render<'a, I: Iterator<Item = Row<'a>>>(
        &self,
        frame: &mut Frame,
        area: Rect,
        content: HistoryContent<'a, I>,
    ) {
        let headers = content.columns.iter().map(|(_, name)| name.as_str());
        let widths = content.columns.iter().map(|(constraint, _)| *constraint);

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
