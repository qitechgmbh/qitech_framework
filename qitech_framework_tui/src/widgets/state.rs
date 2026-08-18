use crossterm::event::KeyCode;
use qitech_framework_core::report::StatePropertyEvent;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Rect;
use ratatui::widgets::Row;

use crate::components::EventLogContent;
use crate::components::EventLogMenu;
use crate::components::EventLogViewAction;
use crate::components::InspectView;
use crate::components::Navigation;
use crate::types::AppAction;
use crate::types::KeyResult;
use crate::types::MachineEntry;
use crate::types::StatePropertyFieldState;
use crate::widgets::machines_view::MachinesContext;
use crate::widgets::tab_view::TabItem;

#[derive(Clone)]
pub enum Mode {
    Navigate(Navigation),
    History((Navigation, EventLogMenu)),
    Inspect((Navigation, EventLogMenu, InspectView)),
}

pub struct StatePage {
    mode: Mode,
}

impl StatePage {
    pub fn new() -> Self {
        Self {
            mode: Mode::Navigate(Default::default()),
        }
    }
}

impl TabItem<MachinesContext> for StatePage {
    fn on_key(&mut self, code: KeyCode, ctx: MachinesContext) -> KeyResult<AppAction> {
        let (mode, result) = match self.mode.clone() {
            Mode::Navigate(navigation) => Self::on_key_navigate(navigation, code, ctx),

            Mode::History((navigation, history)) => {
                Self::on_key_history(navigation, history, code, ctx)
            }

            Mode::Inspect((navigation, history, inspector)) => {
                Self::on_key_inspect(navigation, history, inspector, code)
            }
        };

        self.mode = mode;
        result
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, in_focus: bool, ctx: MachinesContext) {
        let machine = ctx.selected();

        match &mut self.mode {
            Mode::Navigate(navigation) => {
                Self::render_navigate(navigation, frame, area, in_focus, machine);
            }

            Mode::History((_, history)) => {
                Self::render_history(history, frame, area, machine);
            }

            Mode::Inspect((_, _, inspector)) => inspector.render(frame, area),
        }
    }
}

// --- navigate ---
impl StatePage {
    pub fn on_key_navigate(
        mut navigation: Navigation,
        code: KeyCode,
        ctx: MachinesContext,
    ) -> (Mode, KeyResult<AppAction>) {
        let machine = ctx.selected();
        let prop_count = machine.state.len().saturating_sub(1);

        // --- let component handle event ---
        let Err(code) = navigation.on_key(code, prop_count) else {
            return (
                Mode::Navigate(navigation),
                KeyResult::Handled(AppAction::NoAction),
            );
        };

        // -- ensure the value is clamped before reading---
        navigation.clamp(prop_count);

        // --- retrieve and ensure the property is initialized ---
        let (key, field) = ctx
            .selected()
            .state
            .get_index(navigation.pos())
            .expect("failed to read from clamped index");

        match &field.state {
            StatePropertyFieldState::NotInitialized => {
                (Mode::Navigate(navigation), KeyResult::Bubble(code))
            }

            StatePropertyFieldState::Initialized { .. } => match code {
                KeyCode::Char(' ') => (
                    Mode::History((navigation, EventLogMenu::new(key.clone()))),
                    KeyResult::Handled(AppAction::NoAction),
                ),

                code => (Mode::Navigate(navigation), KeyResult::Bubble(code)),
            },
        }
    }

    pub fn render_navigate(
        navigation: &mut Navigation,
        frame: &mut Frame,
        area: Rect,
        in_focus: bool,
        machine: &MachineEntry,
    ) {
        let items = machine.state.iter().map(|(_, field)| {
            let value = match &field.state {
                StatePropertyFieldState::NotInitialized => "N/A".to_string(),
                StatePropertyFieldState::Initialized { value } => value.to_string(),
            };

            (field.label.clone(), value)
        });

        navigation.render(frame, area, items, in_focus);
    }
}

// --- history ---
impl StatePage {
    pub fn on_key_history(
        navigation: Navigation,
        mut history: EventLogMenu,
        code: KeyCode,
        ctx: MachinesContext,
    ) -> (Mode, KeyResult<AppAction>) {
        let field = ctx
            .selected()
            .state
            .get(history.label())
            .expect("selected property dissapeared");

        let records = &field.records;

        // --- let component handle event ---
        match history.on_key(code, records.len().saturating_sub(1)) {
            EventLogViewAction::NoAction => (
                Mode::History((navigation, history)),
                KeyResult::Handled(AppAction::NoAction),
            ),

            EventLogViewAction::Exit => (Mode::Navigate(navigation), KeyResult::Bubble(code)),

            EventLogViewAction::Inspect(pos) => {
                let record = records.get(pos).expect("selected record dissapeared");
                let inspect = InspectView::new(history.label().to_string(), format!("{record:#?}"));

                (
                    Mode::Inspect((navigation, history, inspect)),
                    KeyResult::Bubble(code),
                )
            }

            EventLogViewAction::Bubble(code) => (
                Mode::History((navigation, history)),
                KeyResult::Bubble(code),
            ),
        }
    }

    pub fn render_history(
        history: &mut EventLogMenu,
        frame: &mut Frame,
        area: Rect,
        machine: &MachineEntry,
    ) {
        let field = machine
            .state
            .get(history.label())
            .expect("selected property entry dissapeared");

        let rows = field.records.iter().rev().map(|record| {
            let (event, value) = match &record.event {
                StatePropertyEvent::Registered { value } => {
                    ("Registered".to_string(), format!("value={value}"))
                }

                StatePropertyEvent::ValueChanged { value } => {
                    ("ValueChanged".to_string(), format!("{value}"))
                }
            };

            Row::new([
                record.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                event,
                value,
            ])
        });

        let content = EventLogContent {
            rows,
            cols: vec![
                (Constraint::Length(19), "Timestamp".to_string()),
                (Constraint::Length(32), "Kind".to_string()),
                (Constraint::Min(0), "Info".to_string()),
            ],
        };

        history.render(frame, area, content);
    }
}

// --- inspect ---
impl StatePage {
    pub fn on_key_inspect(
        navigation: Navigation,
        history: EventLogMenu,
        mut inspector: InspectView,
        code: KeyCode,
    ) -> (Mode, KeyResult<AppAction>) {
        // --- let component handle event ---
        let Err(code) = inspector.on_key(code) else {
            return (
                Mode::Inspect((navigation, history, inspector)),
                KeyResult::Handled(AppAction::NoAction),
            );
        };

        match code {
            KeyCode::Esc => (
                Mode::History((navigation, history)),
                KeyResult::Handled(AppAction::NoAction),
            ),

            code => (
                Mode::Inspect((navigation, history, inspector)),
                KeyResult::Bubble(code),
            ),
        }
    }
}
