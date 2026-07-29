use ratatui::prelude::*;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use crate::widgets::AppWidget;
use crate::widgets::AppWidgetState;
use crate::widgets::Menu;
use crate::widgets::AppContext;

pub struct MachinesWidget {

}

impl AppWidget<AppContext> for MachinesWidget {
    fn height(&self) -> Constraint {
        Constraint::Min(0)
    }

    fn display(&self, ctx: &AppContext, state: AppWidgetState, chunk: Rect, frame: &mut Frame) {
        let style = match state {
            AppWidgetState::NoFocus => Style::default(),
            AppWidgetState::InFocus => Style::default().fg(Color::Blue),
            AppWidgetState::Editing => unreachable!(),
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Tabs
                Constraint::Min(0),    // Machine
            ])
            .split(chunk);

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

pub struct MachinesWidget;