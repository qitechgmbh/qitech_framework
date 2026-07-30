use crossterm::event::KeyCode;
use indexmap::IndexMap;
use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
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
use ratatui::widgets::Tabs;

use crate::ConfigField;
use crate::MachineEntry;
use crate::MeasurementField;
use crate::StateField;
use crate::pages::Page;
use crate::types::AppAction;
use crate::types::AppContext;
use crate::types::Focus;

mod nav;
use nav::Cursor;

enum Mode {
    Navigate,
    Edit { value: String, dirty: bool },
}

pub struct MachinesPage {
    machine: Option<usize>,
    cursor: Cursor,
    mode: Mode,
}

impl Page for MachinesPage {
    fn at_top(&self) -> bool {
        matches!(self.cursor, Cursor::Tab)
    }

    fn on_key_event(&mut self, code: KeyCode, ctx: &AppContext) -> AppAction {
        let Some(idx) = self.synced_machine_idx(ctx) else {
            self.cursor = Cursor::Tab;
            self.machine = None;
            return AppAction::NoAction;
        };

        self.machine = Some(idx);
        let machine = &ctx.machines[idx];

        // --- edit mode ---
        if let Mode::Edit { value, dirty } = &mut self.mode {
            return match code {
                KeyCode::Esc => {
                    self.mode = Mode::Navigate;
                    AppAction::NoAction
                }

                KeyCode::Enter => {
                    let value = value.clone();
                    self.mode = Mode::Navigate;

                    match self.cursor {
                        Cursor::Config { field } => {
                            let (key, _) = machine.config.get_index(field).unwrap();

                            AppAction::SetConfig {
                                machine: machine.ident,
                                resource: key.clone(),
                                value,
                            }
                        }

                        _ => AppAction::NoAction,
                    }
                }

                KeyCode::Char(c) => {
                    if !*dirty {
                        value.clear(); // first key replaces original value
                        *dirty = true;
                    }

                    value.push(c);
                    AppAction::NoAction
                }

                KeyCode::Backspace => {
                    *dirty = true;
                    value.pop();
                    AppAction::NoAction
                }

                _ => AppAction::NoAction,
            };
        }

        // --- navigate mode ---
        match code {
            KeyCode::Up => {
                self.cursor.up(machine);
                AppAction::NoAction
            }
            KeyCode::Down => {
                self.cursor.down(machine);
                AppAction::NoAction
            }

            KeyCode::Left => {
                let Some(idx) = self.machine else {
                    return AppAction::NoAction;
                };

                // increment then sync
                self.machine = Some(idx.saturating_sub(1));
                self.machine = self.synced_machine_idx(ctx);

                AppAction::NoAction
            }

            KeyCode::Right => {
                let Some(idx) = self.machine else {
                    return AppAction::NoAction;
                };

                // increment then sync
                self.machine = Some(idx + 1);
                self.machine = self.synced_machine_idx(ctx);

                AppAction::NoAction
            }

            KeyCode::Enter => {
                if let Cursor::Config { field } = self.cursor {
                    // --- start edit mode ---
                    let (_, field) = machine.config.get_index(field).unwrap();

                    // if value is N/A we can't set it
                    let Some(value) = &field.value else {
                        return AppAction::NoAction;
                    };

                    self.mode = Mode::Edit {
                        value: value.to_string(),
                        dirty: false,
                    };
                }

                AppAction::NoAction
            }
            _ => AppAction::NoAction,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect, ctx: &AppContext) {
        const TITLE: &str = "Machines";

        let style = match ctx.focus {
            Focus::Content => Style::default().fg(Color::LightBlue),
            _ => Style::default(),
        };

        let block = Block::default()
            .title(TITLE)
            .borders(Borders::ALL)
            .border_style(style);

        frame.render_widget(&block, area);

        let inner = block.inner(area);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Tabs
                Constraint::Min(0),    // Content
            ])
            .split(inner);

        self.render_tabs(frame, chunks[0], ctx);

        let Some(idx) = self.synced_machine_idx(ctx) else {
            self.render_no_machines(frame, chunks[1]);
            return;
        };

        self.render_machine(frame, chunks[1], ctx, &ctx.machines[idx]);
    }
}

impl MachinesPage {
    pub fn new() -> Self {
        Self {
            machine: None,
            cursor: Cursor::Tab,
            mode: Mode::Navigate,
        }
    }

    fn synced_machine_idx(&self, ctx: &AppContext) -> Option<usize> {
        if ctx.machines.is_empty() {
            None
        } else {
            Some(
                self.machine
                    .unwrap_or(0)
                    .min(ctx.machines.len().saturating_sub(1)),
            )
        }
    }

    fn render_tabs(&self, frame: &mut Frame, area: Rect, ctx: &AppContext) {
        let style = match self.cursor {
            Cursor::Tab if ctx.focus == Focus::Content => Style::default().fg(Color::LightBlue),
            _ => Style::default(),
        };

        let mut titles = Vec::new();
        for entry in ctx.machines {
            titles.push(format!("{} ({})", entry.title, entry.ident.serial));
        }

        let tabs = Tabs::new(titles)
            .select(self.synced_machine_idx(ctx))
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(style),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(tabs, area);
    }

    fn render_no_machines(&self, frame: &mut Frame, chunk: Rect) {
        const PARAGRAPH: &str = "No Machines detected";

        let error = Paragraph::new(PARAGRAPH)
            .alignment(Alignment::Center)
            .block(Block::default());

        let area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .split(chunk)[1];

        frame.render_widget(error, area);
    }

    fn render_machine(
        &self,
        frame: &mut Frame,
        chunk: Rect,
        ctx: &AppContext,
        page: &MachineEntry,
    ) {
        let chunks = Layout::vertical([
            Constraint::Length(2 + page.config.len() as u16),
            Constraint::Length(2 + page.state.len() as u16),
            Constraint::Length(2 + page.measurements.len() as u16),
        ])
        .split(chunk);

        self.render_config(frame, chunks[0], ctx, &page.config);
        self.render_state(frame, chunks[1], ctx, &page.state);
        self.render_measurement(frame, chunks[2], ctx, &page.measurements);
    }

    fn render_config(
        &self,
        frame: &mut Frame,
        chunk: Rect,
        ctx: &AppContext,
        items: &IndexMap<String, ConfigField>,
    ) {
        const TITLE: &str = " Config ";

        let border_style = match ctx.focus {
            Focus::Content if self.cursor.is_config() => Style::default().fg(Color::LightBlue),
            _ => Style::default(),
        };

        let rows: Vec<Row> = items
            .iter()
            .enumerate()
            .map(|(i, (_, field))| {
                let selected = matches!(
                    self.cursor,
                    Cursor::Config { field } if field == i
                );

                let editing = selected && matches!(self.mode, Mode::Edit { .. });

                let style = if editing {
                    Style::default().fg(Color::Red)
                } else if selected {
                    Style::default().fg(Color::LightBlue)
                } else {
                    Style::default()
                };

                let value = if editing {
                    match &self.mode {
                        Mode::Edit { value, .. } => value.clone(),
                        _ => unreachable!(),
                    }
                } else {
                    match &field.value {
                        Some(v) => format!("{v}"),
                        None => "N/A".to_string(),
                    }
                };

                Row::new(vec![Cell::from(field.label.clone()), Cell::from(value)]).style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [Constraint::Percentage(60), Constraint::Percentage(40)],
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(TITLE),
        );

        frame.render_widget(table, chunk);
    }

    fn render_state(
        &self,
        frame: &mut Frame,
        chunk: Rect,
        ctx: &AppContext,
        items: &IndexMap<String, StateField>,
    ) {
        const TITLE: &str = " State ";

        let border_style = match ctx.focus {
            Focus::Content if self.cursor.is_state() => Style::default().fg(Color::LightBlue),
            _ => Style::default(),
        };

        let rows: Vec<Row> = items
            .iter()
            .enumerate()
            .map(|(i, (_, field))| {
                let style = match self.cursor {
                    Cursor::State { field } if field == i => Style::default().fg(Color::LightBlue),
                    _ => Style::default(),
                };

                let value = field
                    .value
                    .as_ref()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "N/A".to_string());

                Row::new(vec![Cell::from(field.label.clone()), Cell::from(value)]).style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [Constraint::Percentage(60), Constraint::Percentage(40)],
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(TITLE),
        );

        frame.render_widget(table, chunk);
    }

    fn render_measurement(
        &self,
        frame: &mut Frame,
        chunk: Rect,
        ctx: &AppContext,
        items: &IndexMap<String, MeasurementField>,
    ) {
        const TITLE: &str = " Measurements ";

        let border_style = match ctx.focus {
            Focus::Content if self.cursor.is_measurement() => Style::default().fg(Color::LightBlue),
            _ => Style::default(),
        };

        // --- measurements ---
        let rows: Vec<Row> = items
            .iter()
            .enumerate()
            .map(|(i, (_, field))| {
                let style = match self.cursor {
                    Cursor::Measurement { field } if field == i => {
                        Style::default().fg(Color::LightBlue)
                    }
                    _ => Style::default(),
                };

                let value = match &field.value {
                    Some(v) => match v {
                        Some(v) => format!("{v:.3}"),
                        None => "null".to_string(),
                    },
                    None => "N/A".to_string(),
                };

                Row::new(vec![Cell::from(field.label.clone()), Cell::from(value)]).style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [Constraint::Percentage(60), Constraint::Percentage(40)],
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(TITLE),
        );

        frame.render_widget(table, chunk);
    }
}
