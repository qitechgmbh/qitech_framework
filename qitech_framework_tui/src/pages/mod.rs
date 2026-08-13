use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::types::AppAction;
mod machines2;
pub use machines2::MachinesPage;

pub struct TabManager<Ctx> {
    active: usize,
    pages: Vec<TabEntry<Ctx>>,
}

impl<Ctx> TabManager<Ctx> {
    pub fn new(pages: Vec<TabEntry<Ctx>>) -> Self {
        assert!(!pages.is_empty(), "Must have atleast one page");
        Self { active: 0, pages }
    }

    pub fn can_enter(&self) -> bool {
        self.pages[self.selected_page_pos()].page.can_enter()
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &TabEntry<Ctx>> {
        self.pages.iter()
    }

    pub fn selected_page_pos(&self) -> usize {
        self.active
    }

    pub fn goto_prev(&mut self) {
        self.active = self.active.saturating_sub(1);
    }

    pub fn goto_next(&mut self) {
        let max = self.pages.len().saturating_sub(1);
        self.active = self.active.saturating_add(1).min(max);
    }

    pub fn goto_page(&mut self, page: usize) {
        assert!(page < self.pages.len());
        self.active = page;
    }

    pub fn page_title(&mut self) -> &str {
        &self.pages[self.active].title
    }

    pub fn on_key(&mut self, code: KeyCode, ctx: Ctx) -> TabAction {
        self.pages[self.active].page.on_key(code, ctx)
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, ctx: Ctx, in_focus: bool) {
        self.pages[self.active]
            .page
            .render(frame, area, ctx, in_focus);
    }
}

pub struct TabEntry<Ctx> {
    pub title: &'static str,
    pub page: Box<dyn TabWidget<Ctx>>,
}

pub trait TabWidget<Ctx> {
    fn can_enter(&self) -> bool;
    fn on_key(&mut self, code: KeyCode, ctx: Ctx) -> TabAction;
    fn render(&self, frame: &mut Frame, area: Rect, ctx: Ctx, in_focus: bool);
}

pub enum TabAction {
    Exit,
    AppAction(AppAction),
}

impl TabAction {
    pub fn no_action() -> Self {
        Self::AppAction(AppAction::NoAction)
    }
}
