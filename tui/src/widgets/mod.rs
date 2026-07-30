use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::types::AppAction;

mod status;
pub use status::StatusDisplay;

mod tab_view;
pub use tab_view::TabView;

mod machines_view;

pub trait Widget<Ctx> {
    fn on_key(&mut self, code: KeyCode, ctx: Ctx) -> Result<AppAction, KeyCode>;
    fn render(&self, frame: &mut Frame, area: Rect, ctx: Ctx, in_focus: bool);
}
