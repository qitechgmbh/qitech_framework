use crossterm::event::KeyCode;
use qitech_framework_core::ScalarValue;
use qitech_framework_core::report::ConfigPropertyEvent;
use qitech_framework_core::report::ConfigPropertyWriteOutcome;
use qitech_framework_core::report::Constraints;
use qitech_framework_core::schema::ScalarPropertyKind;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Rect;
use ratatui::widgets::Row;

use crate::components::EditMenu;
use crate::components::EditorAction;
use crate::components::EventLogContent;
use crate::components::EventLogMenu;
use crate::components::EventLogViewAction;
use crate::components::InspectView;
use crate::components::Navigation;
use crate::types::AppAction;
use crate::types::ConfigFieldState;
use crate::types::KeyResult;
use crate::types::MachineEntry;
use crate::widgets::machines_view::MachinesContext;
use crate::widgets::tab_view::TabItem;

#[derive(Clone)]
pub enum Mode {
    Navigate(Navigation),
    History((Navigation, EventLogMenu)),
    Inspect((Navigation, EventLogMenu, InspectView)),
    Editing((Navigation, EditMenu)),
}

pub struct ConfigPage {
    mode: Mode,
}

impl ConfigPage {
    pub fn new() -> Self {
        Self {
            mode: Mode::Navigate(Default::default()),
        }
    }
}

impl TabItem<MachinesContext> for ConfigPage {
    fn on_key(&mut self, code: KeyCode, ctx: MachinesContext) -> KeyResult<AppAction> {
        let machine = ctx.selected();
        let prop_count = machine.config.len().saturating_sub(1);

        if prop_count == 0 {
            return KeyResult::Bubble(code);
        }

        let (mode, result) = match self.mode.clone() {
            Mode::Navigate(navigation) => Self::on_key_navigate(code, ctx, navigation),

            Mode::History((navigation, history)) => {
                Self::on_key_history(code, ctx, navigation, history)
            }

            Mode::Inspect((navigation, history, inspector)) => {
                Self::on_key_inspect(code, navigation, history, inspector)
            }

            Mode::Editing((navigation, editor)) => {
                Self::on_key_editor(code, ctx, navigation, editor)
            }
        };

        self.mode = mode;
        result
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, in_focus: bool, ctx: MachinesContext) {
        let machine = ctx.selected();

        match &mut self.mode {
            Mode::Navigate(navigation) => {
                Self::render_navigate(navigation, frame, area, in_focus, ctx.selected());
            }

            Mode::History((_, history)) => {
                Self::render_history(history, frame, area, machine);
            }

            Mode::Inspect((_, _, inspector)) => {
                inspector.render(frame, area);
            }

            Mode::Editing((_, editor)) => {
                Self::render_editor(editor, frame, area, machine);
            }
        }
    }
}

// --- navigate ---
impl ConfigPage {
    pub fn on_key_navigate(
        code: KeyCode,
        ctx: MachinesContext,
        mut navigation: Navigation,
    ) -> (Mode, KeyResult<AppAction>) {
        let machine = ctx.selected();
        let prop_count = machine.config.len().saturating_sub(1);

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
            .config
            .get_index(navigation.pos())
            .expect("failed to read from clamped index");

        match &field.state {
            ConfigFieldState::NotInitialized => (
                Mode::Navigate(navigation),
                KeyResult::Handled(AppAction::NoAction),
            ),

            ConfigFieldState::Initialized {
                value, capability, ..
            } => {
                match code {
                    KeyCode::Char(' ') => (
                        Mode::History((navigation, EventLogMenu::new(key.clone()))),
                        KeyResult::Handled(AppAction::NoAction),
                    ),

                    KeyCode::Enter => {
                        if capability.is_forbidden() {
                            // TODO: flash red and display that is forbidden
                            return (
                                Mode::Navigate(navigation),
                                KeyResult::Handled(AppAction::NoAction),
                            );
                        }

                        // --- switch to edit mode ---
                        (
                            Mode::Editing((
                                navigation,
                                EditMenu::new(key.clone(), value.to_string()),
                            )),
                            KeyResult::Handled(AppAction::NoAction),
                        )
                    }

                    code => (Mode::Navigate(navigation), KeyResult::Bubble(code)),
                }
            }
        }
    }

    pub fn render_navigate(
        navigation: &mut Navigation,
        frame: &mut Frame,
        area: Rect,
        in_focus: bool,
        machine: &MachineEntry,
    ) {
        let items = machine.config.iter().map(|(_, field)| {
            let value = match &field.state {
                ConfigFieldState::NotInitialized => "N/A".to_string(),
                ConfigFieldState::Initialized { value, .. } => value.to_string(),
            };

            (field.label.clone(), value)
        });

        navigation.render(frame, area, items, in_focus);
    }
}

// --- history ---
impl ConfigPage {
    pub fn on_key_history(
        code: KeyCode,
        ctx: MachinesContext,
        navigation: Navigation,
        mut history: EventLogMenu,
    ) -> (Mode, KeyResult<AppAction>) {
        let field = ctx
            .selected()
            .config
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
            .config
            .get(history.label())
            .expect("selected property entry dissapeared");

        let rows = field.records.iter().rev().map(|record| {
            let (event, value) = match &record.event {
                ConfigPropertyEvent::Registered {
                    default,
                    capability,
                    constraints,
                } => (
                    "Registered".to_string(),
                    format!("default={default}, write={capability}, constraints={constraints}"),
                ),

                ConfigPropertyEvent::DefaultChanged(value) => {
                    ("DefaultChanged".to_string(), format!("{value}"))
                }

                ConfigPropertyEvent::CapabilityChanged(value) => {
                    ("CapabilityChanged".to_string(), format!("{value}"))
                }

                ConfigPropertyEvent::ConstraintsChanged(constraints) => {
                    ("ConstraintsChanged".to_string(), format!("{constraints}"))
                }

                ConfigPropertyEvent::Written {
                    value,
                    origin,
                    outcome,
                } => {
                    let outcome = match outcome {
                        ConfigPropertyWriteOutcome::Accepted { .. } => "Accepted",
                        ConfigPropertyWriteOutcome::Rejected(_) => "Rejected",
                    };

                    (
                        format!("Written ({origin})"),
                        format!("{value} => {outcome}"),
                    )
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
impl ConfigPage {
    pub fn on_key_inspect(
        code: KeyCode,
        navigation: Navigation,
        history: EventLogMenu,
        mut inspector: InspectView,
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

// --- inspect ---
impl ConfigPage {
    pub fn on_key_editor(
        code: KeyCode,
        ctx: MachinesContext,
        navigation: Navigation,
        mut editor: EditMenu,
    ) -> (Mode, KeyResult<AppAction>) {
        match editor.on_key(code) {
            Ok(EditorAction::NoAction) => (
                Mode::Editing((navigation, editor)),
                KeyResult::Handled(AppAction::NoAction),
            ),

            Ok(EditorAction::Abort) => (
                Mode::Navigate(navigation),
                KeyResult::Handled(AppAction::NoAction),
            ),

            Ok(EditorAction::Submit(value)) => {
                let machine = ctx.selected();
                let action = Self::edit_to_action(machine, editor.label(), value);
                (Mode::Navigate(navigation), KeyResult::Handled(action))
            }

            Err(code) => (Mode::Navigate(navigation), KeyResult::Bubble(code)),
        }
    }

    pub fn render_editor(
        editor: &mut EditMenu,
        frame: &mut Frame,
        area: Rect,
        machine: &MachineEntry,
    ) {
        let field = machine
            .config
            .get(editor.label())
            .expect("Expected assigned property");

        let ConfigFieldState::Initialized { constraints, .. } = &field.state else {
            unreachable!("Cannot enter edit mode with non initialized property");
        };

        // --- constraints ---
        let rows: Vec<Row> = match constraints {
            Constraints::None => vec![Row::default()],

            Constraints::Numeric { min, max } => {
                let mut rows = vec![Row::new(["Nullable".to_string()])];

                rows.push(Row::new([
                    "Min".to_string(),
                    match min {
                        ScalarValue::Null => "null".to_string(),
                        ScalarValue::Integer(value) => format!("{value}"),
                        ScalarValue::Float(value) => format!("{value}"),
                        _ => unreachable!(),
                    },
                ]));

                rows.push(Row::new([
                    "Max".to_string(),
                    match max {
                        ScalarValue::Null => "null".to_string(),
                        ScalarValue::Integer(value) => format!("{value}"),
                        ScalarValue::Float(value) => format!("{value}"),
                        _ => unreachable!(),
                    },
                ]));

                rows
            }

            Constraints::String {
                min_length,
                max_length,
                pattern,
            } => {
                let mut rows = vec![];

                if let Some(min_length) = min_length {
                    rows.push(Row::new(["Min length".to_string(), min_length.to_string()]));
                }

                rows.push(Row::new(["Max length".to_string(), max_length.to_string()]));

                if let Some(pattern) = pattern {
                    rows.push(Row::new(["Pattern".to_string(), pattern.to_string()]));
                }

                rows
            }

            Constraints::Enum { allowed } => {
                vec![Row::new(["Allowed".to_string(), format!("{allowed:?}")])]
            }
        };

        editor.render(frame, area, rows);
    }

    fn edit_to_action(machine: &MachineEntry, key: &str, value: String) -> AppAction {
        let Some(field) = machine.config.get(key) else {
            return AppAction::NoAction;
        };

        let value = match &field.kind {
            ScalarPropertyKind::Enum { variants, .. } => {
                if !variants.contains_name(&value) {
                    return AppAction::NoAction;
                }

                ScalarValue::Enum(value)
            }

            ScalarPropertyKind::String => {
                // TODO: capability check
                ScalarValue::String(value)
            }

            ScalarPropertyKind::Boolean => {
                let value = match value.parse::<bool>() {
                    Ok(v) => v,
                    Err(_) => return AppAction::NoAction,
                };

                ScalarValue::Boolean(value)
            }

            ScalarPropertyKind::Integer => {
                let value = match value.parse::<i64>() {
                    Ok(v) => v,
                    Err(_) => return AppAction::NoAction,
                };

                ScalarValue::Integer(value)
            }

            ScalarPropertyKind::Float { .. } => {
                let value = match value.parse::<f64>() {
                    Ok(v) => v,
                    Err(_) => return AppAction::NoAction,
                };

                ScalarValue::Float(value)
            }
        };

        AppAction::SetConfig {
            machine: machine.ident,
            resource: key.to_string(),
            value,
        }
    }
}
