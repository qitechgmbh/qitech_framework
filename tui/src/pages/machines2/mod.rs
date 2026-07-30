use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Offset;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::symbols;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ratatui::widgets::ListState;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Tabs;

use crate::AppContext;
use crate::MachineEntry;
use crate::pages::TabAction;
use crate::pages::TabManager;
use crate::pages::TabWidget;

mod config;
use config::ConfigPage;

mod state;
use state::StatePage;

#[derive(Clone, Copy)]
struct Context {
    machine: *const MachineEntry,
}

pub struct MachinesPage {
    selected: usize,
    selection: Option<usize>,
    sections: TabManager<Context>,
    in_section: bool,
}

impl Default for MachinesPage {
    fn default() -> Self {
        Self {
            selected: 0,
            selection: None,
            sections: TabManager::new(vec![
                PageEntry {
                    title: "Config",
                    page: Box::new(ConfigPage::default()),
                },
                PageEntry {
                    title: "State",
                    page: Box::new(StatePage::default()),
                },
            ]),
            in_section: false,
        }
    }
}

impl TabWidget<AppContext> for MachinesPage {
    fn can_enter(&self) -> bool {
        true
    }

    fn on_key(&mut self, code: KeyCode, ctx: AppContext) -> TabAction {
        let machines: &[MachineEntry] = unsafe { &*ctx.machines };

        if machines.is_empty() {
            return TabAction::no_action();
        }

        // --- sync index ---
        self.selected = self.selected.min(machines.len() - 1);

        if let Some(s) = self.selection {
            self.selection = match code {
                KeyCode::Esc => None,
                KeyCode::Enter => {
                    // apply selection
                    self.selected = s;
                    None
                }
                KeyCode::Up => Some(s.saturating_sub(1)),
                KeyCode::Down => {
                    let max = ctx.machines.len().saturating_sub(1);
                    Some((s + 1).min(max))
                }
                _ => Some(s),
            };
        } else if self.in_section {
            let ctx = Context {
                machine: &machines[self.selected] as *const MachineEntry,
            };
            match self.sections.on_key(code, ctx) {
                TabAction::Exit => self.in_section = false,
                TabAction::AppAction(action) => return TabAction::AppAction(action),
            }
        } else {
            match code {
                KeyCode::Enter => self.selection = Some(self.selected),
                KeyCode::Up => return TabAction::Exit,
                KeyCode::Down => self.in_section = true,
                _ => {}
            }
        }

        TabAction::no_action()
    }

    fn render(&self, frame: &mut Frame, area: Rect, ctx: AppContext, in_focus: bool) {
        if ctx.machines.is_empty() {
            return;
        }

        let height = if self.selection.is_some() {
            (ctx.machines.len() as u16 + 2).min(area.height)
        } else {
            3
        };

        let widget_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height,
        };

        let border_style = if in_focus && !self.in_section {
            Style::reset().fg(Color::Blue)
        } else {
            Style::reset().fg(Color::White)
        };

        let machines = unsafe { &*ctx.machines };

        if let Some(s) = self.selection {
            let items: Vec<ListItem> = machines
                .iter()
                .enumerate()
                .map(|(i, machine)| {
                    let text = if i == s {
                        format!("{} ({}) <", machine.title, machine.ident.serial)
                    } else {
                        format!("{} ({})", machine.title, machine.ident.serial)
                    };

                    let mut item = ListItem::new(text);

                    if i == s {
                        item = item.style(
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        );
                    }

                    item
                })
                .collect();

            let list = List::new(items).block(
                Block::default()
                    .title(" Machine ")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            );

            let mut state = ListState::default();
            state.select(Some(self.selected));

            frame.render_stateful_widget(list, widget_area, &mut state);
        } else {
            let selected = &machines[self.selected];

            let selector =
                Paragraph::new(format!("{} ({}) v", selected.title, selected.ident.serial))
                    .block(
                        Block::default()
                            .title(" Machine ")
                            .borders(Borders::ALL)
                            .border_style(border_style),
                    )
                    .style(Style::reset());

            frame.render_widget(selector, widget_area);
        }

        // --- draw border ---
        let section_area = Rect {
            x: area.x,
            y: area.y + height,
            width: area.width,
            height: area.height.saturating_sub(height),
        };

        let block = Block::bordered().border_style(border_style);
        let inner = block.inner(section_area);
        frame.render_widget(block, section_area);

        // --- draw tabs ---
        let titles = self
            .sections
            .iter()
            .map(|t| Line::from(t.title))
            .collect::<Vec<_>>();

        let tabs = Tabs::new(titles)
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .select(self.sections.selected_page_pos())
            .divider(symbols::DOT)
            .padding(" ", " ")
            .style(border_style);

        frame.render_widget(tabs, section_area + Offset::new(1, 0));

        // --- finally draw the content ---
        let machine = &machines[self.selected] as *const MachineEntry;
        let ctx = Context { machine };
        self.sections
            .render(frame, inner, ctx, in_focus && self.in_section);
    }
}
