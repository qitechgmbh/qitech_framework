use qitech_framework_common::RuntimeStatus;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;

use crate::types::AppContext;
use crate::types::Focus;

pub struct StatusWidget;

impl StatusWidget {
    pub fn render(&self, frame: &mut Frame, area: Rect, ctx: &AppContext) {
        const TITLE: &str = "Status";

        let status = match ctx.rt_status {
            RuntimeStatus::Offline => " Offline ",
            RuntimeStatus::DiscoveringEtherCATInterface => " Discovering EtherCAT Interface",
            RuntimeStatus::InitializingEtherCAT => " Initializing EtherCAT ",
            RuntimeStatus::InitializinhModbus => " Initializing Modbus ",
            RuntimeStatus::BuildingMachines => " Building Machines ",
            RuntimeStatus::FinalizingEtherCAT => " Finalizing EtherCAT ",
            RuntimeStatus::Initialized => " Initialized ",
            RuntimeStatus::Running { in_pre_op } => {
                if in_pre_op {
                    " Running (Pre-Op) "
                } else {
                    " Running "
                }
            }
        };

        let style = match ctx.focus {
            Focus::Status => Style::default().fg(Color::Blue),
            _ => Style::default(),
        };

        let text = format!("Runtime: {}", status);
        let info = Paragraph::new(text).block(
            Block::default()
                .title(TITLE)
                .borders(Borders::ALL)
                .border_style(style),
        );

        frame.render_widget(info, area);
    }
}
