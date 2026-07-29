use ratatui::prelude::*;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Tabs;

use crate::app::App;
use crate::app::RuntimeStatus;
use crate::app::TabPosition;
use crate::app::VerticalPosition;
use crate::pages::Page;
use crate::styles;

impl App {
    pub fn display(&self, frame: &mut Frame) {
        const TITLE: &str = " QiTech Control (Terminal Edition) ";

        let outer = Block::default().borders(Borders::ALL).title(TITLE);

        let inner = outer.inner(frame.area());
        frame.render_widget(outer, frame.area());

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Info
                Constraint::Length(3), // Tabs
                Constraint::Min(0),    // Content
            ])
            .split(inner);

        self.draw_status(frame, chunks[0]);
        self.draw_tabs(frame, chunks[1]);
        self.draw_page(frame, chunks[2]);
    }

    fn draw_status(&self, frame: &mut Frame, chunk: Rect) {
        let runtime_status = match self.runtime_status {
            RuntimeStatus::Offline => "🔴 Offline",
            RuntimeStatus::Starting => "🟡 Starting",
            RuntimeStatus::Running => "🟢 Running",
        };

        let border_style = match self.pos_v {
            VerticalPosition::Status => styles::on_hover(),
            _ => Style::default(),
        };

        let info = Paragraph::new(format!("Runtime: {}", runtime_status,)).block(
            Block::default()
                .title(" Status ")
                .borders(Borders::ALL)
                .border_style(border_style),
        );

        frame.render_widget(info, chunk);
    }

    fn draw_tabs(&self, frame: &mut Frame, chunk: Rect) {
        const TITLE: &str = " Menu ";

        let titles = Self::TABS
            .iter()
            .map(|t| Line::from(*t))
            .collect::<Vec<_>>();

        let style = match self.pos_v {
            VerticalPosition::Tab => styles::on_hover(),
            _ => Style::default(),
        };

        let tabs = Tabs::new(titles)
            .select(self.pos_t as usize)
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

    fn draw_page(&self, frame: &mut Frame, chunk: Rect) {
        let title = match self.pos_t {
            TabPosition::Machines => " Machines ",
            TabPosition::EtherCAT => " EtherCAT ",
            TabPosition::Modbus => " Modbus ",
            TabPosition::Logs => " Logs ",
        };

        let style = match self.pos_v {
            VerticalPosition::Page => styles::on_hover(),
            _ => Style::default(),
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(style);

        let inner = block.inner(chunk);
        frame.render_widget(block, chunk);

        #[allow(clippy::single_match)]
        match self.pos_t {
            TabPosition::Machines => self.page_machines.display(frame, inner),
            _ => {}
        }
    }
}
