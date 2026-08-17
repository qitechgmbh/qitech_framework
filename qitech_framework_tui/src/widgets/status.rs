use qitech_framework_core::report::EtherCATStatus;
use qitech_framework_core::report::RuntimeInitStatus;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;

use crate::types::AppContext;
use crate::types::RuntimeStatus;

#[derive(Default)]
pub struct StatusDisplay;

impl StatusDisplay {
    pub fn render(&self, frame: &mut Frame, area: Rect, in_focus: bool, ctx: AppContext) {
        const TITLE: &str = "Status";

        let style = if in_focus {
            Style::reset().fg(Color::Blue)
        } else {
            Style::reset()
        };

        let text = format!(
            "Runtime:  {}\nEtherCAT: {}",
            Self::status_runtime(ctx),
            Self::status_ethercat(ctx),
        );

        let info = Paragraph::new(text).block(
            Block::default()
                .title(TITLE)
                .borders(Borders::ALL)
                .border_style(style),
        );

        frame.render_widget(info, area);
    }

    fn status_runtime(ctx: AppContext) -> &'static str {
        match ctx.rt_status {
            RuntimeStatus::Offline => "🔴 Offline",

            RuntimeStatus::Initializing(status) => match status {
                RuntimeInitStatus::NotStarted => "🟡 Not Started",
                RuntimeInitStatus::EtherCATDiscovery => "🟡 EtherCAT Discovery",
                RuntimeInitStatus::EtherCATInitializingDevices => {
                    "🟡 Initializing EtherCAT Devices"
                }
                RuntimeInitStatus::ModbusRTUDiscovery => "🟡 Modbus RTU Discovery",
                RuntimeInitStatus::XtremDiscovery => "🟡 XTREM Discovery",
                RuntimeInitStatus::BuildingMachines => "🟡 Building Machines",
                RuntimeInitStatus::Finalizing => "🟡 Finalizing",
                RuntimeInitStatus::Completed => "🔵 Initialization Completed",
                RuntimeInitStatus::Failed => "🔴 Initialization Failed",
            },

            RuntimeStatus::Running => "🟢 Running",
            RuntimeStatus::Disconnected => "🔴 Disconnected",
        }
    }

    fn status_ethercat(ctx: AppContext) -> &'static str {
        match ctx.ecat_status {
            EtherCATStatus::NoInterface => "🔴 No Interface",
            EtherCATStatus::Boot => "🟡 Boot",
            EtherCATStatus::Init => "🟡 Init",
            EtherCATStatus::PreOp => "🔵 PreOp",
            EtherCATStatus::PreopPdi => "🟡 PreopPdi",
            EtherCATStatus::Op => "🟢 Op",
        }
    }
}
