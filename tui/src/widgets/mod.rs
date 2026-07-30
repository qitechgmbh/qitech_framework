use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;

use crate::types::AppAction;

mod status;
pub use status::StatusWidget;

mod tab_view;
pub use tab_view::TabView;

mod drop_down;
pub use drop_down::DropDown;

mod machines_view;

pub struct WidgetManager<Ctx> {
    selected: usize,
    widgets: Vec<Box<dyn Widget<Ctx>>>,
}

impl<Ctx: Copy> WidgetManager<Ctx> {
    pub fn new(widgets: Vec<Box<dyn Widget<Ctx>>>) -> Self {
        assert!(!widgets.is_empty(), "Must have atleast one widget");

        Self {
            selected: 0,
            widgets,
        }
    }

    pub fn on_key_event(&mut self, code: KeyCode, ctx: Ctx) -> AppAction {
        match self.widgets[self.selected].on_key(code, ctx) {
            WidgetAction::NoAction => AppAction::NoAction,
            WidgetAction::GotoPrev => {
                self.selected = self.selected.saturating_sub(1);
                AppAction::NoAction
            }
            WidgetAction::GotoNext => {
                let max = self.widgets.len().saturating_sub(1);
                self.selected = self.selected.saturating_add(1).min(max);
                AppAction::NoAction
            }
            WidgetAction::AppAction(action) => action,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, ctx: Ctx) {
        let constraints: Vec<Constraint> = self
            .widgets
            .iter()
            .map(|widget| widget.constraint())
            .collect();

        let chunks = Layout::vertical(constraints).split(area);

        for (i, (entry, chunk)) in self.widgets.iter().zip(chunks.iter()).enumerate() {
            let focused = i == self.selected;
            entry.render(frame, *chunk, ctx, focused);
        }
    }
}

pub trait Widget<Ctx: Copy> {
    fn constraint(&self) -> Constraint;
    fn on_key(&mut self, code: KeyCode, ctx: Ctx) -> WidgetAction;
    fn render(&self, frame: &mut Frame, area: Rect, ctx: Ctx, in_focus: bool);
}

pub enum WidgetAction {
    NoAction,
    GotoPrev,
    GotoNext,
    AppAction(AppAction),
}
