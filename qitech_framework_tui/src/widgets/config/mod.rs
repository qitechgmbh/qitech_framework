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
use crate::components::HistoryAction;
use crate::components::HistoryContent;
use crate::components::HistoryMenu;
use crate::components::InspectMenu;
use crate::components::Navigation;
use crate::types::AppAction;
use crate::types::ConfigFieldState;
use crate::types::MachineEntry;
use crate::widgets::machines_view::MachinesContext;
use crate::widgets::tab_view::TabItem;

#[derive(Default)]
pub struct ConfigPage {
    mode: Mode,
    navigation: Navigation,
}

impl TabItem<MachinesContext> for ConfigPage {
    fn on_key(&mut self, code: KeyCode, ctx: MachinesContext) -> Result<AppAction, KeyCode> {
        let machine = unsafe { &*ctx.selected };
        let prop_count = machine.config.len().saturating_sub(1);

        if prop_count == 0 {
            return Err(code);
        }

        match &mut self.mode {
            Mode::Navigate => {
                // --- let component handle event ---
                let Err(code) = self.navigation.on_key(code, prop_count) else {
                    return Ok(AppAction::NoAction);
                };

                // -- ensure the value is clamped before reading---
                self.navigation.clamp(prop_count);

                // retrieve and ensure the property is initialized
                let (key, field) = machine
                    .config
                    .get_index(self.navigation.pos())
                    .expect("failed to read from clamped index");

                match &field.state {
                    ConfigFieldState::NotInitialized => Ok(AppAction::NoAction),
                    ConfigFieldState::Initialized {
                        value, capability, ..
                    } => {
                        match code {
                            KeyCode::Char(' ') => {
                                self.mode = Mode::History(HistoryMenu::new(key.clone()))
                            }

                            KeyCode::Enter => {
                                if capability.is_forbidden() {
                                    // TODO: flash red and display that is forbidden
                                    return Ok(AppAction::NoAction);
                                }

                                // --- switch mode ---
                                self.mode =
                                    Mode::Editing(EditMenu::new(key.clone(), value.to_string()));
                            }

                            other => return Err(other),
                        }

                        Ok(AppAction::NoAction)
                    }
                }
            }

            Mode::History(menu) => {
                let field = machine
                    .config
                    .get(menu.label())
                    .expect("selected property dissapeared");

                let records = &field.records;
                let limit = records.len().saturating_sub(1);

                // --- let component handle event ---
                match menu.on_key(code, limit) {
                    HistoryAction::NoAction => {}
                    HistoryAction::Exit => self.mode = Mode::Navigate,
                    HistoryAction::Inspect(pos) => {
                        let record = records.get(pos).expect("selected record dissapeared");

                        let menu =
                            InspectMenu::new(menu.label().to_string(), format!("{record:#?}"));

                        self.mode = Mode::Inspect(menu);
                    }
                    HistoryAction::Bubble(code) => return Err(code),
                }

                Ok(AppAction::NoAction)
            }

            Mode::Inspect(inspector) => {
                // --- let component handle event ---
                let Err(code) = inspector.on_key(code) else {
                    return Ok(AppAction::NoAction);
                };

                match code {
                    KeyCode::Esc => {
                        self.mode = Mode::History(HistoryMenu::new(inspector.label().to_string()));
                        Ok(AppAction::NoAction)
                    }

                    code => Err(code),
                }
            }

            Mode::Editing(editor) => match editor.on_key(code) {
                Ok(EditorAction::Yield(value)) => {
                    let action = Self::edit_to_action(machine, editor.label(), value);
                    self.mode = Mode::Navigate;
                    Ok(action)
                }
                Ok(EditorAction::NoAction) => Ok(AppAction::NoAction),
                Ok(EditorAction::Abort) => {
                    self.mode = Mode::Navigate;
                    Ok(AppAction::NoAction)
                }
                Err(code) => Err(code),
            },
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect, ctx: MachinesContext, in_focus: bool) {
        let machine = unsafe { &*ctx.selected };

        match &self.mode {
            Mode::Navigate => {
                let items = machine.config.iter().map(|(_, field)| {
                    let value = match &field.state {
                        ConfigFieldState::NotInitialized => "N/A".to_string(),
                        ConfigFieldState::Initialized { value, .. } => value.to_string(),
                    };

                    (field.label.clone(), value)
                });

                self.navigation.render(frame, area, items, in_focus);
            }

            Mode::History(menu) => {
                let field = machine
                    .config
                    .get(menu.label())
                    .expect("selected property entry dissapeared");

                let rows = field.records.iter().rev().map(|record| {
                    let (event, value) = match &record.event {
                        ConfigPropertyEvent::Registered {
                            default,
                            capability,
                            constraints,
                        } => (
                            "Registered".to_string(),
                            format!("default={default}, write={capability:?}, constraints={constraints:?}"),
                        ),

                        ConfigPropertyEvent::DefaultChanged(value) => {
                            ("DefaultChanged".to_string(), format!("{value}"))
                        }

                        ConfigPropertyEvent::CapabilityChanged(value) => {
                            ("CapabilityChanged".to_string(), format!("{value}"))
                        }

                        ConfigPropertyEvent::ConstraintsChanged(value) => (
                            "ConstraintsChanged".to_string(),
                            match value {
                                Constraints::None => "None".to_string(),

                                Constraints::Numeric { min, max } => {
                                    format!("[{min}..{max}]")
                                }

                                Constraints::String {
                                    min_length,
                                    max_length,
                                    pattern,
                                } => {
                                    match pattern {
                                        Some(pattern) => format!("[{min_length}..{max_length}] / {pattern}"),
                                        None => format!("[{min_length}..{max_length}]"),
                                    }
                                }

                                Constraints::Enum { allowed } => {
                                    format!("[{}]", allowed.join(", "))
                                }
                            },
                        ),

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

                let content = HistoryContent {
                    rows,
                    columns: vec![
                        (Constraint::Length(19), "Timestamp".to_string()),
                        (Constraint::Length(32), "Kind".to_string()),
                        (Constraint::Min(0), "Info".to_string()),
                    ],
                };

                menu.render(frame, area, content);
            }
            Mode::Inspect(inspector) => inspector.render(frame, area),
            Mode::Editing(editor) => {
                let machine = unsafe { &*ctx.selected };

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
        }
    }
}

impl ConfigPage {
    // -- util ---
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

// --- types ---
#[derive(Default)]
pub enum Mode {
    #[default]
    Navigate,
    History(HistoryMenu),
    Inspect(InspectMenu),
    Editing(EditMenu),
}
