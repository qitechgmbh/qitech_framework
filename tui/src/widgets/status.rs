use crossterm::event::KeyCode;
use qitech_framework_common::RuntimeStatus;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;

use crate::types::AppContext;
use crate::widgets::Widget;
use crate::widgets::WidgetAction;

pub struct StatusWidget;

impl Widget<AppContext> for StatusWidget {
    fn on_key(&mut self, code: KeyCode, ctx: AppContext) -> WidgetAction {
        _ = ctx;

        match code {
            KeyCode::Up => WidgetAction::GotoPrev,
            KeyCode::Down => WidgetAction::GotoNext,
            _ => WidgetAction::NoAction,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect, ctx: AppContext, in_focus: bool) {
        const TITLE: &str = "Status";

        let status = match ctx.rt_status {
            RuntimeStatus::Offline => "🔴 Offline",
            RuntimeStatus::DiscoveringEtherCATInterface => "🟡 Discovering EtherCAT Interface",
            RuntimeStatus::InitializingEtherCAT => "🟡 Initializing EtherCAT",
            RuntimeStatus::DiscoveringModbusDevices => "🟡 Discovering Modbus RTU Devices",
            RuntimeStatus::BuildingMachines => "🟡 Building Machines",
            RuntimeStatus::FinalizingEtherCAT => "🟡 Finalizing EtherCAT",
            RuntimeStatus::Initialized => "🟢 Initialized",
            RuntimeStatus::Running { in_pre_op } => {
                if in_pre_op {
                    "🔵 Running (Pre-Op)"
                } else {
                    "🟢 Running"
                }
            }
        };

        let style = if in_focus {
            Style::default().fg(Color::Blue)
        } else {
            Style::default()
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

    fn constraint(&self) -> Constraint {
        Constraint::Length(3)
    }
}
