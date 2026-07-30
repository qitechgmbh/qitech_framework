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

use crate::MachineEntry;
use crate::controls::DropDown;
use crate::types::AppAction;
use crate::types::AppContext;
use crate::widgets::TabView;
use crate::widgets::Widget;

#[derive(Clone, Copy)]
struct Context {
    machine: *const MachineEntry,
}

enum Focus {
    MachineSelect,
    MachineTabs,
}

pub struct MachinesView {
    focus: Focus,
    drop_down: DropDown,
    machines: TabView<Context>,
}

impl MachinesView {
    pub fn new() -> Self {
        Self {
            focus: Focus::MachineSelect,
            drop_down: DropDown::new("machine"),
            machines: TabView::new(Vec::new()),
        }
    }
}

impl Widget<AppContext> for MachinesView {
    fn on_key(&mut self, code: KeyCode, ctx: AppContext) -> Result<AppAction, KeyCode> {
        let machines: &[MachineEntry] = unsafe { &*ctx.machines };

        if machines.is_empty() {
            return Err(code);
        }

        match self.focus {
            Focus::MachineSelect => {
                let limit = machines.len().saturating_sub(1);

                match self.drop_down.on_key(code, limit) {
                    Ok(()) => Ok(AppAction::NoAction),

                    Err(KeyCode::Down) => {
                        self.focus = Focus::MachineTabs;
                        Ok(AppAction::NoAction)
                    }

                    Err(k) => Err(k),
                }
            }

            Focus::MachineTabs => {
                let machine = &machines[self.drop_down.selected()] as *const MachineEntry;
                let ctx = Context { machine };

                match self.machines.on_key(code, ctx) {
                    Ok(action) => Ok(action),

                    Err(KeyCode::Up) => {
                        self.focus = Focus::MachineSelect;
                        Ok(AppAction::NoAction)
                    }

                    Err(k) => Err(k),
                }
            }
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect, ctx: AppContext, in_focus: bool) {}
}
