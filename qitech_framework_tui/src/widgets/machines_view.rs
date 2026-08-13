use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;

use crate::controls::DropDown;
use crate::types::AppAction;
use crate::types::AppContext;
use crate::types::MachineEntry;
use crate::widgets::TabView;
use crate::widgets::command::CommandsView;
use crate::widgets::config::ConfigPage;
use crate::widgets::events::EventsView;
use crate::widgets::measurements::MeasurementsView;
use crate::widgets::state::StateView;
use crate::widgets::subscriptions::SubscriptionsView;
use crate::widgets::tab_view::TabEntry;
use crate::widgets::tab_view::TabItem;

#[derive(Clone, Copy)]
pub struct MachinesContext {
    pub selected: *const MachineEntry,
    pub machines: *const [MachineEntry],
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Picker,
    Tabs,
}

pub struct MachinesView {
    focus: Focus,
    drop_down: DropDown,
    machines: TabView<MachinesContext>,
}

impl MachinesView {
    pub fn new() -> Self {
        let config = TabEntry {
            title: "Config",
            item: Box::new(ConfigPage::default()),
        };

        let state = TabEntry {
            title: "State",
            item: Box::new(StateView::default()),
        };

        let measurements = TabEntry {
            title: "Measurements",
            item: Box::new(MeasurementsView::default()),
        };

        let commands = TabEntry {
            title: "Commands",
            item: Box::new(CommandsView::default()),
        };

        let events = TabEntry {
            title: "Events",
            item: Box::new(EventsView::default()),
        };

        let subscriptions = TabEntry {
            title: "Subscriptions",
            item: Box::new(SubscriptionsView::new()),
        };

        Self {
            focus: Focus::Picker,
            drop_down: DropDown::new("machine"),
            machines: TabView::new(
                true,
                vec![config, state, measurements, commands, events, subscriptions],
            ),
        }
    }
}

impl TabItem<AppContext> for MachinesView {
    fn on_key(&mut self, code: KeyCode, ctx: AppContext) -> Result<AppAction, KeyCode> {
        let machines: &[MachineEntry] = unsafe { &*ctx.machines };

        if machines.is_empty() {
            return Err(code);
        }

        match self.focus {
            Focus::Picker => {
                let limit = machines.len().saturating_sub(1);

                match self.drop_down.on_key(code, limit) {
                    Ok(()) => Ok(AppAction::NoAction),

                    Err(KeyCode::Down) => {
                        self.focus = Focus::Tabs;
                        Ok(AppAction::NoAction)
                    }

                    Err(k) => Err(k),
                }
            }

            Focus::Tabs => {
                let machine = &machines[self.drop_down.selected()] as *const MachineEntry;
                let ctx = MachinesContext {
                    selected: machine,
                    machines,
                };

                match self.machines.on_key(code, ctx) {
                    Ok(action) => Ok(action),

                    Err(KeyCode::Up) => {
                        self.focus = Focus::Picker;
                        Ok(AppAction::NoAction)
                    }

                    Err(k) => Err(k),
                }
            }
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect, ctx: AppContext, in_focus: bool) {
        let machines = unsafe { &*ctx.machines };

        let items: Vec<String> = machines
            .iter()
            .map(|machine| format!("{} ({})", machine.title.as_str(), machine.ident.serial))
            .collect();

        let drop_down_height = self.drop_down.rendered_height(&items);

        let chunks = Layout::vertical([
            Constraint::Length(drop_down_height as u16),
            Constraint::Min(0),
        ])
        .split(area);

        // Render dropdown
        self.drop_down.render(
            frame,
            chunks[0],
            in_focus && self.focus == Focus::Picker,
            &items,
        );

        if machines.is_empty() {
            return;
        }

        let selected = &machines[self.drop_down.selected()] as *const MachineEntry;
        let machines_ctx = MachinesContext { selected, machines };

        // Render remaining area
        self.machines.render(
            frame,
            chunks[1],
            machines_ctx,
            in_focus && self.focus == Focus::Tabs,
        );
    }
}
