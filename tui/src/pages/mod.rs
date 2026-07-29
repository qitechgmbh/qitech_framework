mod machines;
use crossterm::event::KeyCode;
pub use machines::MachinesPage;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::types::AppAction;
use crate::types::AppContext;

pub struct ContentManager {
    active: PageId,
    machines: MachinesPage,
}

impl ContentManager {
    pub fn new() -> Self {
        Self {
            active: PageId::Machines,
            machines: MachinesPage::new(),
        }
    }

    pub fn selected_id(&self) -> PageId {
        self.active
    }

    pub fn goto_page(&mut self, page: PageId) {
        self.active = page;
    }

    pub fn at_top(&self) -> bool {
        match self.active {
            PageId::Machines => self.machines.at_top(),
            PageId::EtherCAT => todo!(),
            PageId::Modbus => todo!(),
            PageId::Logs => todo!(),
        }
    }

    pub fn on_key_event(&mut self, code: KeyCode, ctx: &AppContext) -> AppAction {
        match self.active {
            PageId::Machines => self.machines.on_key_event(code, ctx),
            PageId::EtherCAT => todo!(),
            PageId::Modbus => todo!(),
            PageId::Logs => todo!(),
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, ctx: &AppContext) {
        #[allow(clippy::single_match)]
        match self.active {
            PageId::Machines => self.machines.render(frame, area, ctx),
            _ => {}
        }
    }
}

// --- traits ---
trait Page {
    fn at_top(&self) -> bool;
    fn on_key_event(&mut self, code: KeyCode, ctx: &AppContext) -> AppAction;
    fn render(&self, frame: &mut Frame, area: Rect, ctx: &AppContext);
}

// --- types ---
#[derive(Clone, Copy, Default)]
#[repr(u8)]
pub enum PageId {
    #[default]
    Machines,
    EtherCAT,
    Modbus,
    Logs,
}
