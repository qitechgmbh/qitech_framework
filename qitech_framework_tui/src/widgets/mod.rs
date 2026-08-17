use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;

mod status;
pub use status::StatusDisplay;

pub mod tab_view;
pub use tab_view::TabView;

pub mod command;
pub mod config;
pub mod events;
mod logs;
pub mod measurements;
pub mod state;
pub mod subscriptions;
pub mod transactions;

use crate::types::AppAction;

mod machines_view;
pub use machines_view::MachinesView;

pub trait Widget<Ctx> {
    fn on_key(&mut self, code: KeyCode, ctx: Ctx) -> Result<AppAction, KeyCode>;
    fn render(&self, frame: &mut Frame, area: Rect, ctx: Ctx, in_focus: bool);
}
