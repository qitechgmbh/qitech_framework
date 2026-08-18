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
use ratatui::widgets::TableState;

use crate::types::AppAction;
use crate::types::KeyResult;
use crate::widgets::machines_view::MachinesContext;
use crate::widgets::tab_view::TabItem;

#[derive(Default)]
pub enum Mode {
    #[default]
    Navigate,
    History(usize),
    Inspect(usize),
}

#[derive(Default)]
pub struct EventsView {
    selected: usize,
    mode: Mode,
}

impl TabItem<MachinesContext> for EventsView {
    fn on_key(&mut self, code: KeyCode, ctx: MachinesContext) -> KeyResult<AppAction> {
        match self.mode {
            Mode::Navigate => self.on_key_navigate(code, ctx),
            Mode::History(pos) => self.on_key_history(code, ctx, pos),
            Mode::Inspect(pos) => self.on_key_inspect(code, pos),
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, in_focus: bool, ctx: MachinesContext) {
        _ = in_focus;

        match self.mode {
            Mode::Navigate => self.render_navigate(frame, area, ctx, in_focus),
            Mode::History(pos) => self.render_history(frame, area, ctx, pos),
            Mode::Inspect(pos) => self.render_inspect(frame, area, ctx, pos),
        }
    }
}

// --- navigate ---
impl EventsView {
    fn on_key_navigate(&mut self, code: KeyCode, ctx: MachinesContext) -> KeyResult<AppAction> {
        let machine = unsafe { &*ctx.selected };

        match code {
            KeyCode::Up => {
                if self.selected == 0 {
                    return KeyResult::Bubble(code);
                }

                self.selected -= 1;
            }

            KeyCode::Down => {
                let max = machine.events.len().saturating_sub(1);
                self.selected = (self.selected + 1).min(max);
            }

            KeyCode::Char(' ') => {
                self.mode = Mode::History(0);
            }

            _ => return KeyResult::Bubble(code),
        }

        KeyResult::Handled(AppAction::NoAction)
    }

    fn render_navigate(&self, frame: &mut Frame, area: Rect, ctx: MachinesContext, in_focus: bool) {
        let machine = unsafe { &*ctx.selected };

        let visible = area.height as usize;
        let total = machine.events.len();

        let offset = if total <= visible {
            0
        } else {
            self.selected
                .saturating_sub(visible / 2)
                .min(total - visible)
        };

        let rows: Vec<Row> = machine
            .events
            .iter()
            .skip(offset)
            .take(visible)
            .enumerate()
            .map(|(visible_index, (_, field))| {
                let index = offset + visible_index;

                let style = if index == self.selected && in_focus {
                    Style::default().fg(Color::LightBlue)
                } else {
                    Style::default()
                };

                Row::new(vec![Cell::from(field.label.as_str())]).style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [Constraint::Percentage(60), Constraint::Percentage(40)],
        )
        .style(Style::default());

        frame.render_widget(table, area);
    }
}

// --- history ---
impl EventsView {
    fn on_key_history(
        &mut self,
        code: KeyCode,
        ctx: MachinesContext,
        pos: usize,
    ) -> KeyResult<AppAction> {
        let machine = unsafe { &*ctx.selected };

        let (_, field) = machine.events.get_index(self.selected).unwrap();

        match code {
            KeyCode::Esc => {
                self.mode = Mode::Navigate;
            }

            KeyCode::Char(' ') => {
                self.mode = Mode::Inspect(pos);
            }

            // don't consume exit button
            KeyCode::Char('q') => return KeyResult::Bubble(code),

            KeyCode::Up => {
                self.mode = Mode::History(pos.saturating_sub(1));
            }

            KeyCode::Down => {
                let max = field.records.len().saturating_sub(1);
                self.mode = Mode::History((pos + 1).min(max));
            }

            _ => {}
        }

        KeyResult::Handled(AppAction::NoAction)
    }

    fn render_history(&self, frame: &mut Frame, area: Rect, ctx: MachinesContext, pos: usize) {
        let machine = unsafe { &*ctx.selected };

        let (name, field) = machine
            .events
            .get_index(self.selected)
            .expect("Selected invalid item");

        let rows = field
            .records
            .iter()
            .rev()
            .map(|record| Row::new([record.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()]));

        let table = Table::new(rows, [Constraint::Min(1)])
            .header(Row::new(["Timestamp"]).style(Style::default().add_modifier(Modifier::BOLD)))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Events ({name}) "))
                    .border_style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .column_spacing(4)
            .row_highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        let mut state = TableState::default();
        state.select(Some(pos));

        frame.render_stateful_widget(table, area, &mut state);
    }
}

// --- inspect ---
impl EventsView {
    fn on_key_inspect(&mut self, code: KeyCode, pos: usize) -> KeyResult<AppAction> {
        match code {
            KeyCode::Esc => {
                self.mode = Mode::History(pos);
                KeyResult::Handled(AppAction::NoAction)
            }

            // don't consume exit button
            KeyCode::Char('q') => KeyResult::Bubble(code),

            // consume other keys
            _ => KeyResult::Handled(AppAction::NoAction),
        }
    }

    fn render_inspect(&self, frame: &mut Frame, area: Rect, ctx: MachinesContext, pos: usize) {
        let machine = unsafe { &*ctx.selected };

        let (name, field) = machine
            .events
            .get_index(self.selected)
            .expect("Selected invalid item");

        let text = field
            .records
            .iter()
            .rev()
            .nth(pos)
            .map(|record| format!("{record:#?}"))
            .unwrap_or_else(|| "Invalid record".into());

        let paragraph = Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Inspect ({name}) "))
                .border_style(
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
        );

        frame.render_widget(paragraph, area);
    }
}
