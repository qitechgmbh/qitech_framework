use ratatui::prelude::*;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Tabs;
use crate::widgets::AppWidget;
use crate::widgets::AppWidgetState;
use crate::widgets::AppContext;

pub struct MenuWidget;

impl MenuWidget {
    pub const TABS: [&'static str; 4] = ["Machines", "EtherCAT", "Modbus", "Logs"];
}

impl AppWidget for MenuWidget {
    fn height(&self) -> Constraint {
        Constraint::Length(3)
    }

    fn display(&self, shared: &AppContext, state: AppWidgetState, chunk: Rect, frame: &mut Frame) {
        const TITLE: &str = " Menu ";

        let titles = Self::TABS
            .iter()
            .map(|t| Line::from(*t))
            .collect::<Vec<_>>();

        let style = match state {
            AppWidgetState::NoFocus => Style::default(),
            AppWidgetState::InFocus => Style::default().fg(Color::Blue),
            AppWidgetState::Editing => unreachable!(),
        };

        let tabs = Tabs::new(titles)
            .select(shared.selected_menu)
            .block(
                Block::default()
                    .title(TITLE)
                    .borders(Borders::ALL)
                    .border_style(style),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(tabs, chunk);
    }
}
