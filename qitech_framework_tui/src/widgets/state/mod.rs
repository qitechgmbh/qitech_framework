use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::types::AppAction;
use crate::widgets::machines_view::MachinesContext;
use crate::widgets::tab_view::TabItem;

mod history;
mod inspect;
mod navigate;

#[derive(Default)]
pub enum Mode {
    #[default]
    Navigate,
    History(usize),
    Inspect(usize),
}

#[derive(Default)]
pub struct StateView {
    selected: usize,
    mode: Mode,
}

impl TabItem<MachinesContext> for StateView {
    fn on_key(&mut self, code: KeyCode, ctx: MachinesContext) -> Result<AppAction, KeyCode> {
        match self.mode {
            Mode::Navigate => self.on_key_navigate(code, ctx),
            Mode::History(pos) => self.on_key_history(code, ctx, pos),
            Mode::Inspect(pos) => self.on_key_inspect(code, pos),
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect, ctx: MachinesContext, in_focus: bool) {
        match self.mode {
            Mode::Navigate => self.render_navigate(frame, area, ctx, in_focus),
            Mode::History(pos) => self.render_history(frame, area, ctx, pos),
            Mode::Inspect(pos) => self.render_inspect(frame, area, ctx, pos),
        }
    }
}
