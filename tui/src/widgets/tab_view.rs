use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Offset;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::symbols;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::Tabs;

use crate::pages::TabAction;
use crate::pages::TabEntry;
use crate::pages::TabManager;
use crate::types::AppContext;
use crate::widgets::Widget;
use crate::widgets::WidgetAction;

pub struct TabView<Ctx: Copy> {
    manager: TabManager<Ctx>,
    in_tab: bool,
}

impl<Ctx: Copy> TabView<Ctx> {
    pub fn new(pages: Vec<TabEntry<Ctx>>) -> Self {
        Self {
            manager: TabManager::new(pages),
            in_tab: false,
        }
    }
}

impl<Ctx: Copy> Widget<Ctx> for TabView<Ctx> {
    fn on_key(&mut self, code: KeyCode, ctx: Ctx) -> WidgetAction {
        _ = ctx;

        if self.in_tab {
            match self.manager.on_key(code, ctx) {
                TabAction::Exit => {
                    self.in_tab = false;
                    WidgetAction::NoAction
                }
                TabAction::AppAction(action) => WidgetAction::AppAction(action),
            }
        } else {
            match code {
                KeyCode::Left => {
                    self.manager.goto_prev();
                    WidgetAction::NoAction
                }
                KeyCode::Right => {
                    self.manager.goto_next();
                    WidgetAction::NoAction
                }
                KeyCode::Up => WidgetAction::GotoPrev,
                KeyCode::Down => {
                    if self.manager.can_enter() {
                        self.in_tab = true;
                        WidgetAction::NoAction
                    } else {
                        WidgetAction::GotoNext
                    }
                }
                _ => WidgetAction::NoAction,
            }
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect, ctx: Ctx, in_focus: bool) {
        let border_style = if in_focus && !self.in_tab {
            Style::default().fg(Color::Blue)
        } else {
            Style::default()
        };

        // --- draw border ---
        let block = Block::bordered().border_style(border_style);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        // --- draw tabs ---
        let titles = self
            .manager
            .iter()
            .map(|t| Line::from(t.title))
            .collect::<Vec<_>>();

        let tabs = Tabs::new(titles)
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .select(self.manager.selected_page_pos())
            .divider(symbols::DOT)
            .padding(" ", " ")
            .style(border_style);

        frame.render_widget(tabs, area + Offset::new(1, 0));

        self.manager
            .render(frame, inner, ctx, in_focus && self.in_tab);
    }

    fn constraint(&self) -> Constraint {
        Constraint::Min(0)
    }
}

pub struct ContentWidgetContext {
    app: *const AppContext,
}
