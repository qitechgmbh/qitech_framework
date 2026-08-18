use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;

use crate::types::AppAction;
use crate::types::KeyResult;
use crate::widgets::machines_view::MachinesContext;
use crate::widgets::tab_view::TabItem;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum Mode {
    #[default]
    Navigate,
    Select(usize),
}

pub struct SubscriptionsView {
    mode: Mode,
    selected: usize,
}

impl TabItem<MachinesContext> for SubscriptionsView {
    fn on_key(&mut self, code: KeyCode, ctx: MachinesContext) -> KeyResult<AppAction> {
        match self.mode {
            Mode::Navigate => self.on_key_navigation(code, ctx),
            Mode::Select(_) => self.on_key_select(code, ctx),
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, in_focus: bool, ctx: MachinesContext) {
        match self.mode {
            Mode::Navigate => self.render_navigation(frame, area, ctx, in_focus),
            Mode::Select(_) => self.render_select(frame, area, ctx, in_focus),
        }
    }
}

impl SubscriptionsView {
    pub fn new() -> Self {
        Self {
            mode: Default::default(),
            selected: 0,
        }
    }
}

// --- navigation ---
impl SubscriptionsView {
    fn on_key_navigation(&mut self, code: KeyCode, ctx: MachinesContext) -> KeyResult<AppAction> {
        let machine = unsafe { &*ctx.selected };

        match code {
            KeyCode::Char('+') => {
                self.mode = Mode::Select(0);
            }

            KeyCode::Char('-') => {
                let Some(provider) = machine.subscriptions.get_index(self.selected) else {
                    return KeyResult::Handled(AppAction::NoAction);
                };

                return KeyResult::Handled(AppAction::Unsubscribe {
                    provider: *provider,
                    subscriber: machine.ident,
                });
            }

            KeyCode::Up => {
                if self.selected == 0 {
                    return KeyResult::Bubble(code);
                }

                self.selected -= 1;
            }

            KeyCode::Down => {
                if self.selected == machine.subscriptions.len().saturating_sub(1) {
                    return KeyResult::Bubble(code);
                }

                self.selected += 1;
            }

            _ => return KeyResult::Bubble(code),
        }

        KeyResult::Handled(AppAction::NoAction)
    }

    fn render_navigation(
        &self,
        frame: &mut Frame,
        area: Rect,
        ctx: MachinesContext,
        in_focus: bool,
    ) {
        let machine = unsafe { &*ctx.selected };
        let machines = unsafe { &*ctx.machines };

        // --- subscriptions list ---
        let subscription_items: Vec<ListItem> = machine
            .subscriptions
            .iter()
            .enumerate()
            .map(|(i, subscription)| {
                let selected = i == self.selected;

                let style = if selected && in_focus {
                    Style::reset().fg(Color::LightBlue)
                } else {
                    Style::reset()
                };

                let entry = machines.iter().find(|x| x.ident == *subscription).unwrap();

                ListItem::new(format!("{} ({})", entry.title.as_str(), entry.ident.serial))
                    .style(style)
            })
            .collect();

        let list = List::new(subscription_items);
        frame.render_widget(list, area);
    }
}

// --- select ---
impl SubscriptionsView {
    fn on_key_select(&mut self, code: KeyCode, ctx: MachinesContext) -> KeyResult<AppAction> {
        let selected = unsafe { &*ctx.selected };
        let machines = unsafe { &*ctx.machines };

        let candidates: Vec<_> = machines
            .iter()
            .filter(|machine| {
                machine.ident != selected.ident
                    && !selected
                        .subscriptions
                        .iter()
                        .any(|ident| *ident == machine.ident)
            })
            .collect();

        let pos = match self.mode {
            Mode::Select(v) => v.min(candidates.len()),
            Mode::Navigate => unreachable!(),
        };

        match code {
            KeyCode::Enter => {
                let provider = candidates[pos].ident;
                self.mode = Mode::Navigate;

                return KeyResult::Handled(AppAction::Subscribe {
                    provider,
                    subscriber: selected.ident,
                });
            }

            KeyCode::Esc => {
                self.mode = Mode::Navigate;
            }

            KeyCode::Up => {
                if pos == 0 {
                    return KeyResult::Bubble(code);
                }

                self.mode = Mode::Select(pos - 1);
            }

            KeyCode::Down => {
                if pos == candidates.len().saturating_sub(1) {
                    return KeyResult::Bubble(code);
                }

                self.mode = Mode::Select(pos + 1);
            }

            _ => return KeyResult::Bubble(code),
        }

        KeyResult::Handled(AppAction::NoAction)
    }

    fn render_select(&self, frame: &mut Frame, area: Rect, ctx: MachinesContext, in_focus: bool) {
        let selected = unsafe { &*ctx.selected };
        let machines = unsafe { &*ctx.machines };

        let candidates: Vec<_> = machines
            .iter()
            .filter(|machine| {
                machine.ident != selected.ident
                    && !selected
                        .subscriptions
                        .iter()
                        .any(|ident| *ident == machine.ident)
            })
            .collect();

        let selected_index = match self.mode {
            Mode::Select(index) => Some(index.min(candidates.len().saturating_sub(1))),
            Mode::Navigate => None,
        };

        let items: Vec<ListItem> = candidates
            .iter()
            .enumerate()
            .map(|(i, machine)| {
                let style = if Some(i) == selected_index && in_focus {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default()
                };

                ListItem::new(format!(
                    "{} ({})",
                    machine.title.as_str(),
                    machine.ident.serial
                ))
                .style(style)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .title(" Select ")
                .title_style(Style::default().fg(Color::Red))
                .borders(Borders::all())
                .border_style(Style::default().fg(Color::Red)),
        );

        frame.render_widget(list, area);
    }
}
