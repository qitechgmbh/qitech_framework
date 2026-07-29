use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Tabs;

use crate::AppContext;
use crate::pages::PageId;
use crate::types::AppAction;
use crate::types::Focus;

pub struct MenuWidget;

impl MenuWidget {
    pub fn on_key_event(&mut self, code: KeyCode, ctx: &AppContext) -> AppAction {
        match code {
            KeyCode::Left => match ctx.page {
                PageId::Machines => AppAction::NoAction,
                PageId::EtherCAT => AppAction::GotoPage(PageId::Machines),
                PageId::Modbus => AppAction::GotoPage(PageId::EtherCAT),
                PageId::Logs => AppAction::GotoPage(PageId::Modbus),
            },
            KeyCode::Right => match ctx.page {
                PageId::Machines => AppAction::GotoPage(PageId::EtherCAT),
                PageId::EtherCAT => AppAction::GotoPage(PageId::Modbus),
                PageId::Modbus => AppAction::GotoPage(PageId::Logs),
                PageId::Logs => AppAction::NoAction,
            },
            _ => AppAction::NoAction,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, ctx: &AppContext) {
        const ITEMS: [&str; 4] = ["Machines", "EtherCAT", "Modbus", "Logs"];
        const TITLE: &str = " Menu ";

        let titles = ITEMS.iter().map(|t| Line::from(*t)).collect::<Vec<_>>();

        let style = match ctx.focus {
            Focus::Menu => Style::default().fg(Color::Blue),
            _ => Style::default(),
        };

        let index = match ctx.page {
            PageId::Machines => 0,
            PageId::EtherCAT => 1,
            PageId::Modbus => 2,
            PageId::Logs => 3,
        };

        let tabs = Tabs::new(titles)
            .select(index)
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

        frame.render_widget(tabs, area);
    }
}
