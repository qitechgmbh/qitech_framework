use std::vec;

use ratatui::prelude::*;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use crate::widgets::AppWidget;
use crate::widgets::AppWidgetState;
use crate::widgets::Menu;
use crate::widgets::AppContext;

// each component is isolated -> Widget can return children

pub struct ContentRoot {
    items: Vec<(&'static str, Box<dyn AppWidget<AppContext>>)>,
}

impl ContentRoot {
    pub fn new() -> Self {
        Self { 
            items: vec![
                // insert machines
            ],
        }
    }
}

impl AppWidget<AppContext> for ContentRoot {
    fn height(&self) -> Constraint {
        Constraint::Min(0)
    }

    fn display(&self, ctx: &AppContext, chunk: Rect, frame: &mut Frame) {
        let title = match ctx.selected_menu {
            Menu::Machines => " Machines ",
            Menu::EtherCAT => " EtherCAT ",
            Menu::Modbus => " Modbus ",
            Menu::Logs => " Logs ",
        };

        let style = match state {
            AppWidgetState::NoFocus => Style::default(),
            AppWidgetState::InFocus => Style::default().fg(Color::Blue),
            AppWidgetState::Editing => unreachable!(),
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(style);

        frame.render_widget(&block, chunk);
        let inner = block.inner(chunk);

        match shared.selected_menu {
            Menu::Machines => todo!(),
            Menu::EtherCAT => todo!(),
            Menu::Modbus => todo!(),
            Menu::Logs => todo!(),
        }
    }
}
