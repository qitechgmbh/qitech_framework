use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Cell;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Row;
use ratatui::widgets::Table;

use crate::types::AppAction;
use crate::types::AppContext;
use crate::widgets::tab_view::TabItem;

#[derive(Default, PartialEq)]
enum Mode {
    #[default]
    Navigate,
    Inspect,
}

pub struct TransactionsView {
    mode: Mode,
    selected: usize,
}

impl TransactionsView {
    pub fn new() -> Self {
        Self {
            mode: Mode::Navigate,
            selected: 0,
        }
    }
}

impl TabItem<AppContext> for TransactionsView {
    fn on_key(&mut self, code: KeyCode, ctx: AppContext) -> Result<AppAction, KeyCode> {
        match self.mode {
            Mode::Navigate => self.on_key_navigate(code, ctx),
            Mode::Inspect => self.on_key_inspect(code, ctx),
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect, ctx: AppContext, in_focus: bool) {
        match self.mode {
            Mode::Navigate => self.render_navigate(frame, area, ctx, in_focus),
            Mode::Inspect => self.render_inspect(frame, area, ctx, in_focus),
        }
    }
}

// --- navigate ---
impl TransactionsView {
    fn on_key_navigate(&mut self, code: KeyCode, ctx: AppContext) -> Result<AppAction, KeyCode> {
        let transactions = unsafe { &*ctx.transactions };

        match code {
            KeyCode::Up => {
                if self.selected == 0 {
                    return Err(code);
                }

                self.selected = self.selected.saturating_sub(1);
            }

            KeyCode::Down => {
                let max = transactions.len().saturating_sub(1);
                self.selected = (self.selected + 1).min(max);
            }

            KeyCode::Char(' ') => {
                self.mode = Mode::Inspect;
            }

            _ => return Err(code),
        }

        Ok(AppAction::NoAction)
    }

    fn render_navigate(&self, frame: &mut Frame, area: Rect, ctx: AppContext, in_focus: bool) {
        let transactions = unsafe { &*ctx.transactions };

        let header = Row::new([
            Cell::from("ID"),
            Cell::from("Timestamp"),
            Cell::from("Request"),
            Cell::from("Result"),
        ])
        .style(Style::default().add_modifier(Modifier::BOLD));

        let rows = transactions.iter().enumerate().rev().map(|(i, t)| {
            let selected = transactions.len().saturating_sub(1 + self.selected);
            let result = if t.result.is_err() {
                "Failure"
            } else {
                "Success"
            };

            let mut row = Row::new([
                Cell::from(t.id.to_string()),
                Cell::from(t.timestamp.to_string()),
                Cell::from(t.request.to_string()),
                Cell::from(result),
            ]);

            if in_focus && i == selected {
                row = row.style(
                    Style::default()
                        .fg(Color::LightBlue)
                        .add_modifier(Modifier::BOLD),
                );
            }

            row
        });

        let table = Table::new(
            rows,
            [
                Constraint::Length(4),
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ],
        )
        .header(header);

        frame.render_widget(table, area);
    }
}

// --- inspect ---
impl TransactionsView {
    fn on_key_inspect(&mut self, code: KeyCode, ctx: AppContext) -> Result<AppAction, KeyCode> {
        _ = ctx;

        if matches!(code, KeyCode::Esc) || matches!(code, KeyCode::Char(' ')) {
            self.mode = Mode::Navigate;
        }

        Ok(AppAction::NoAction)
    }

    fn render_inspect(&self, frame: &mut Frame, area: Rect, ctx: AppContext, in_focus: bool) {
        _ = in_focus;

        let transactions = unsafe { &*ctx.transactions };
        let selected = transactions.len().saturating_sub(1 + self.selected);

        if let Some(selected) = transactions.get(selected) {
            let text = format!("{:#?}", selected);

            let paragraph = Paragraph::new(text).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Inspecting")
                    .border_style(
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
            );

            frame.render_widget(paragraph, area);
        }
    }
}
