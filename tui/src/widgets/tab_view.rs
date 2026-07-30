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
use ratatui::widgets::Tabs;

use crate::types::AppAction;

pub struct TabView<Ctx: Copy> {
    tabs: Vec<TabEntry<Ctx>>,
    focus: Focus,
    selected_tab: usize,
}

impl<Ctx: Copy> TabView<Ctx> {
    pub fn new(tabs: Vec<TabEntry<Ctx>>) -> Self {
        Self {
            tabs,
            focus: Focus::Tabs,
            selected_tab: 0,
        }
    }
}

impl<Ctx: Copy> TabView<Ctx> {
    pub fn on_key(&mut self, code: KeyCode, ctx: Ctx) -> Result<AppAction, KeyCode> {
        if self.tabs.is_empty() {
            return Err(code);
        }

        if self.focus == Focus::Tabs {
            return match code {
                KeyCode::Left if self.selected_tab > 0 => {
                    self.selected_tab -= 1;
                    Ok(AppAction::NoAction)
                }

                KeyCode::Right if self.selected_tab < (self.tabs.len() - 1) => {
                    self.selected_tab += 1;
                    Ok(AppAction::NoAction)
                }

                KeyCode::Down => {
                    self.focus = Focus::Content;
                    return Ok(AppAction::NoAction);
                }

                _ => Err(code),
            };
        }

        // focus on content
        match self.tabs[self.selected_tab].item.on_key(code, ctx) {
            Ok(v) => return Ok(v),
            Err(k) => match k {
                KeyCode::Up => {
                    self.focus = Focus::Tabs;
                    Ok(AppAction::NoAction)
                }

                _ => Err(k),
            },
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, ctx: Ctx, in_focus: bool) {
        let border_style = if in_focus && self.focus == Focus::Tabs {
            Style::reset().fg(Color::Blue)
        } else {
            Style::reset()
        };

        // --- draw border ---
        let block = Block::bordered().border_style(border_style);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if self.tabs.is_empty() {
            frame.render_widget(Line::from("No tabs available"), inner);
            return;
        }

        // --- draw tabs ---
        let titles = self
            .tabs
            .iter()
            .map(|t| Line::from(t.title))
            .collect::<Vec<_>>();

        let tabs = Tabs::new(titles)
            .highlight_style(
                Style::reset()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .select(self.selected_tab)
            .divider(symbols::DOT)
            .padding(" ", " ")
            .style(border_style);

        frame.render_widget(tabs, area + Offset::new(1, 0));

        self.tabs[self.selected_tab].item.render(
            frame,
            inner,
            ctx,
            in_focus && self.focus == Focus::Content,
        );
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Tabs,
    Content,
}

pub struct TabEntry<Ctx> {
    pub title: &'static str,
    pub item: Box<dyn TabItem<Ctx>>,
}

pub trait TabItem<Ctx> {
    fn on_key(&mut self, code: KeyCode, ctx: Ctx) -> Result<AppAction, KeyCode>;
    fn render(&self, frame: &mut Frame, area: Rect, ctx: Ctx, in_focus: bool);
}
