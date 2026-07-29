// mod run;
// pub use run::run;

use crossterm::event::KeyCode;
use qitech_framework_common::RuntimeStatus;
use ratatui::{Frame, layout::{Constraint, Direction, Layout, Rect}, widgets::{Block, Borders, Paragraph}};

pub struct App {
    // --- state ---
    focus: Focus,

    rt_status: RuntimeStatus,

    // --- components ---
    status: StatusWidget,
}

impl App {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            focus: Focus::Status,
            rt_status: RuntimeStatus::Initialized,
            status: StatusWidget,
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        const TITLE: &str = " QiTech Control (Terminal Edition) ";

        let outer = Block::default().borders(Borders::ALL).title(TITLE);
        frame.render_widget(&outer, frame.area());

        let inner = outer.inner(frame.area());
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Info
                Constraint::Length(3), // Menu
                Constraint::Min(0),    // Content
            ])
            .split(inner);

        self.status.render(chunks[0], frame, self.rt_status);
    }

    pub fn on_key_event(&mut self, code: KeyCode) {
        _ = code;
    }
}

struct StatusWidget;

impl StatusWidget {
    fn render(&self, area: Rect, frame: &mut Frame, status: RuntimeStatus) {
        let text = match status {
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
        
        frame.render_widget(Paragraph::new(text), area);
    }
}

enum Focus {
    Status,
    Menu,
    Content(ContentState),
}

enum ContentState {
    Machines(MachinesState),
    EtherCAT,
    Modbus,
    Logs,
}

struct MachinesState {
    machine: usize,
    section: Section,
    field: usize,
    editing: Option<String>,
}

enum Section {
    Tab,
    Config,
    State,
    Measurement,
}