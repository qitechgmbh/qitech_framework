use crossterm::event::KeyCode;
use qitech_framework_core::report::EtherCATStatus;
use qitech_framework_core::report::RuntimeInitStatus;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;

use crate::types::AppAction;
use crate::types::AppContext;
use crate::types::RuntimeStatus;
use crate::widgets::Widget;

pub struct StatusDisplay;

impl Widget<AppContext> for StatusDisplay {
    fn on_key(&mut self, code: KeyCode, ctx: AppContext) -> Result<AppAction, KeyCode> {
        _ = ctx;

        // forward all events
        Err(code)
    }

    fn render(&self, frame: &mut Frame, area: Rect, ctx: AppContext, in_focus: bool) {
        const TITLE: &str = "Status";

        let runtime = match ctx.rt_status {
            RuntimeStatus::Offline => "🔴 Offline",

            RuntimeStatus::Initializing(status) => match status {
                RuntimeInitStatus::NotStarted => "🟡 Not Started",
                RuntimeInitStatus::EtherCATDiscovery => "🟡 EtherCAT Discovery",
                RuntimeInitStatus::EtherCATInitializingDevices => {
                    "🟡 Initializing EtherCAT Devices"
                }
                RuntimeInitStatus::ModbusRTUDiscovery => "🟡 Modbus RTU Discovery",
                RuntimeInitStatus::BuildingMachines => "🟡 Building Machines",
                RuntimeInitStatus::Finalizing => "🟡 Finalizing",
                RuntimeInitStatus::Completed => "🔵 Initialization Completed",
                RuntimeInitStatus::Failed => "🔴 Initialization Failed",
            },

            RuntimeStatus::Running => "🟢 Running",
            RuntimeStatus::Disconnected => "🔴 Disconnected",
        };

        let ethercat = match ctx.ecat_status {
            EtherCATStatus::NoInterface => "🔴 No Interface",
            EtherCATStatus::Boot => "🟡 Boot",
            EtherCATStatus::Init => "🟡 Init",
            EtherCATStatus::PreOp => "🔵 PreOp",
            EtherCATStatus::PreopPdi => "🟡 PreopPdi",
            EtherCATStatus::Op => "🟢 Op",
        };

        let style = if in_focus {
            Style::default().fg(Color::Blue)
        } else {
            Style::default()
        };

        let text = format!("Runtime:  {}\nEtherCAT: {}", runtime, ethercat);

        let info = Paragraph::new(text).block(
            Block::default()
                .title(TITLE)
                .borders(Borders::ALL)
                .border_style(style),
        );

        frame.render_widget(info, area);
    }
}
