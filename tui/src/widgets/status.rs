use ratatui::prelude::*;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;

use crate::app::RuntimeStatus;
use crate::widgets::AppWidget;
use crate::widgets::AppWidgetState;
use crate::widgets::AppContext;

pub struct StatusWidget;

impl AppWidget for StatusWidget {
    fn height(&self) -> Constraint {
        Constraint::Length(3)
    }

    fn display(&self, shared: &AppContext, state: AppWidgetState, chunk: Rect, frame: &mut Frame) {
        const TITLE: &str = "Runtime: {}";

        let runtime_status = match shared.runtime_status {
            RuntimeStatus::Offline => "🔴 Offline",
            RuntimeStatus::Starting => "🟡 Starting",
            RuntimeStatus::Running => "🟢 Running",
        };

        let style = match state {
            AppWidgetState::NoFocus => Style::default(),
            AppWidgetState::InFocus => Style::default().fg(Color::Blue),
            AppWidgetState::Editing => unreachable!(),
        };

        let info = Paragraph::new(format!("Runtime: {}", runtime_status,)).block(
            Block::default()
                .title(TITLE)
                .borders(Borders::ALL)
                .border_style(style),
        );

        frame.render_widget(info, chunk);
    }
}
