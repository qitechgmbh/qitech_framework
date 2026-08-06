use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::types::AppAction;
use crate::widgets::machines_view::MachinesContext;
use crate::widgets::tab_view::TabItem;

#[derive(Default)]
pub struct EventsView {
    selected: usize,
}

impl TabItem<MachinesContext> for EventsView {
    fn on_key(&mut self, code: KeyCode, ctx: MachinesContext) -> Result<AppAction, KeyCode> {
        Err(code)
    }

    fn render(&self, frame: &mut Frame, area: Rect, ctx: MachinesContext, in_focus: bool) {}
}
