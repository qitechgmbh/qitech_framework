use crossterm::event::KeyCode;
use qitech_framework_core::request::RuntimeRequestKind;
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
use crate::types::KeyResult;
use crate::widgets::tab_view::TabItem;

#[derive(Default, PartialEq)]
enum Mode {
    #[default]
    Navigate,
    Inspect,
}

struct Entry {
    // ID
    // Timestamp
    // Request
    // Result
}

pub struct TransactionsPage {
    mode: Mode,
    selected: usize,
    entries: Vec<Entry>,
}

impl TransactionsPage {
    pub fn new() -> Self {
        Self {
            mode: Mode::Navigate,
            entries: Vec::default(),
            selected: 0,
        }
    }
}

impl TabItem<AppContext> for TransactionsPage {
    fn on_key(&mut self, code: KeyCode, ctx: AppContext) -> KeyResult<AppAction> {
        _ = ctx;

        match self.mode {
            Mode::Navigate => self.on_key_navigate(code),
            Mode::Inspect => self.on_key_inspect(code),
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, in_focus: bool, ctx: AppContext) {
        match self.mode {
            Mode::Navigate => self.render_navigate(frame, area, ctx, in_focus),
            Mode::Inspect => self.render_inspect(frame, area, ctx, in_focus),
        }
    }
}

// --- navigate ---
impl TransactionsPage {
    fn on_key_navigate(&mut self, code: KeyCode) -> KeyResult<AppAction> {
        match code {
            KeyCode::Up if self.selected > 0 => {
                self.selected -= 1;
            }

            KeyCode::Down => {
                let limit = self.entries.len().saturating_sub(1);
                self.selected = (self.selected + 1).min(limit);
            }

            KeyCode::Char(' ') => {
                self.mode = Mode::Inspect;
            }

            _ => return KeyResult::Bubble(code),
        }

        KeyResult::Handled(AppAction::NoAction)
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

            let request = match &t.request {
                RuntimeRequestKind::WriteMachineDeviceInfo {
                    machine_ident,
                    role,
                    subdevice_index,
                } => {
                    format!(
                        "WriteMachineDeviceInfo({}, {}, {})",
                        machine_ident, role, subdevice_index
                    )
                }

                RuntimeRequestKind::SetConfigProperty {
                    target,
                    path,
                    value,
                } => {
                    format!("SetConfigProperty({}, {}, {})", target, path, value)
                }

                RuntimeRequestKind::ExecuteCommand { target, path } => {
                    format!("ExecuteCommand({}, {})", target, path)
                }

                RuntimeRequestKind::SubscribeMachine {
                    provider,
                    subscriber,
                } => {
                    format!("SubscribeMachine({}, {})", provider, subscriber)
                }

                RuntimeRequestKind::UnsubscribeMachine {
                    provider,
                    subscriber,
                } => {
                    format!("UnsubscribeMachine({}, {})", provider, subscriber)
                }
            };

            let mut row = Row::new([
                Cell::from(t.id.to_string()),
                Cell::from(t.timestamp.to_string()),
                Cell::from(request),
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
                Constraint::Fill(2),
                Constraint::Fill(3),
                Constraint::Fill(1),
            ],
        )
        .header(header);

        frame.render_widget(table, area);
    }
}

// --- inspect ---
impl TransactionsPage {
    fn on_key_inspect(&mut self, code: KeyCode) -> KeyResult<AppAction> {
        if matches!(code, KeyCode::Esc) || matches!(code, KeyCode::Char(' ')) {
            self.mode = Mode::Navigate;
        }

        KeyResult::Handled(AppAction::NoAction)
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
