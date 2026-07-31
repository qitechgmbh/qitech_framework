use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;

use crate::types::AppContext;
use crate::widgets::StatusDisplay;
use crate::widgets::Widget;

pub struct UIRoot {
    status: StatusDisplay,
}

impl UIRoot {
    pub fn new() -> Self {
        Self { status: StatusDisplay }
    }

    pub fn render(&self, frame: &mut Frame, ctx: AppContext) {
        const TITLE: &str = " QiTech Control (Terminal Edition) ";

        let outer = Block::default().borders(Borders::ALL).title(TITLE);
        frame.render_widget(&outer, frame.area());

        let inner = outer.inner(frame.area());
        // self.widgets.render(frame, inner, self.as_context());

        self.status.render(frame, inner, ctx, true);
    }

    pub fn on_key(&mut self, event: KeyEvent) -> Result<(), KeyEvent> {
        Err(event)
    }
}
