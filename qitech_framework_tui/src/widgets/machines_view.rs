use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;

use crate::controls::DropDown;
use crate::types::AppAction;
use crate::types::AppContext;
use crate::types::KeyResult;
use crate::types::MachineEntry;
use crate::widgets::TabView;
use crate::widgets::command::CommandsView;
use crate::widgets::config::ConfigPage;
use crate::widgets::events::EventsView;
use crate::widgets::measurements::MeasurementsPage;
use crate::widgets::state::StatePage;
use crate::widgets::subscriptions::SubscriptionsView;
use crate::widgets::tab_view::TabEntry;
use crate::widgets::tab_view::TabItem;

#[derive(Clone, Copy)]
pub struct MachinesContext {
    pub selected: *const MachineEntry,
    pub machines: *const [MachineEntry],
}

impl MachinesContext {
    pub fn selected(&self) -> &MachineEntry {
        unsafe { &*self.selected }
    }

    // pub fn machines(&self) -> &[MachineEntry] {
    //     unsafe { &*self.machines }
    // }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Picker,
    Tabs,
}

pub struct MachinesPage {
    focus: Focus,
    drop_down: DropDown,
    tabs: TabView<MachinesContext>,
}

impl MachinesPage {
    pub fn new() -> Self {
        let config = TabEntry {
            title: "Config",
            item: Box::new(ConfigPage::new()),
        };

        let state = TabEntry {
            title: "State",
            item: Box::new(StatePage::new()),
        };

        let measurements = TabEntry {
            title: "Measurements",
            item: Box::new(MeasurementsPage::new()),
        };

        let commands = TabEntry {
            title: "Commands",
            item: Box::new(CommandsView::new()),
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
            drop_down: DropDown::new("machine".to_string()),
            tabs: TabView::new(
                true,
                vec![config, state, measurements, commands, events, subscriptions],
            ),
        }
    }
}

impl TabItem<AppContext> for MachinesPage {
    fn on_key(&mut self, code: KeyCode, ctx: AppContext) -> KeyResult<AppAction> {
        if ctx.machines.is_empty() {
            return KeyResult::Bubble(code);
        }

        let ctx_tab = MachinesContext {
            selected: &ctx.machines()[self.drop_down.selected()] as *const MachineEntry,
            machines: ctx.machines(),
        };

        match self.focus {
            Focus::Picker => match self.drop_down.on_key(code) {
                KeyResult::Handled(_) => KeyResult::Handled(AppAction::NoAction),

                KeyResult::Bubble(KeyCode::Down) => {
                    self.focus = Focus::Tabs;
                    KeyResult::Handled(AppAction::NoAction)
                }

                KeyResult::Bubble(k) => KeyResult::Bubble(k),
            },

            Focus::Tabs => match self.tabs.on_key(code, ctx_tab) {
                KeyResult::Handled(action) => KeyResult::Handled(action),

                KeyResult::Bubble(KeyCode::Up) => {
                    self.focus = Focus::Picker;
                    KeyResult::Handled(AppAction::NoAction)
                }

                KeyResult::Bubble(k) => KeyResult::Bubble(k),
            },
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, in_focus: bool, ctx: AppContext) {
        let machines = ctx.machines();

        // --- update drop down ---
        let options: Vec<String> = machines
            .iter()
            .map(|machine| format!("{} ({})", machine.title.as_str(), machine.ident.serial))
            .collect();

        let chunks = Layout::vertical([
            Constraint::Length(self.drop_down.rendered_height(&options) as u16),
            Constraint::Min(0),
        ])
        .split(area);

        // --- render dropdown ---
        self.drop_down.render(
            frame,
            chunks[0],
            in_focus && self.focus == Focus::Picker,
            options,
        );

        // --- render remaining area ---
        if machines.is_empty() {
            return;
        }

        let machine = &machines[self.drop_down.selected()] as *const MachineEntry;

        let ctx_tab = MachinesContext {
            selected: machine,
            machines,
        };

        self.tabs.render(
            frame,
            chunks[1],
            in_focus && self.focus == Focus::Tabs,
            ctx_tab,
        );
    }
}
