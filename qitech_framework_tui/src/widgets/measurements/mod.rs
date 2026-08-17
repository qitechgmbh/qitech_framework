use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::components::Navigation;
use crate::types::AppAction;
use crate::widgets::machines_view::MachinesContext;
use crate::widgets::tab_view::TabItem;

mod graph;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum Mode {
    #[default]
    Navigate,
    Graph,
}

#[derive(Default)]
pub struct MeasurementsView {
    mode: Mode,
    navigation: Navigation,

    // --- graph state ---
    zoom: u8,
    offset: f64,
}

impl TabItem<MachinesContext> for MeasurementsView {
    fn on_key(&mut self, code: KeyCode, ctx: MachinesContext) -> Result<AppAction, KeyCode> {
        let machine = unsafe { &*ctx.selected };
        let prop_count = machine.measurements.len().saturating_sub(1);

        if prop_count == 0 {
            return Err(code);
        }

        match self.mode {
            Mode::Navigate => {
                // --- let component handle event ---
                let Err(code) = self.navigation.on_key(code, prop_count) else {
                    return Ok(AppAction::NoAction);
                };

                // --- check if it's a space bar event
                if let KeyCode::Char(' ') = code {
                    // -- ensure the value is clamped before reading---
                    self.navigation.clamp(prop_count);

                    // retrieve and ensure the property is initialized
                    let (_, field) = machine
                        .measurements
                        .get_index(self.navigation.pos())
                        .expect("failed to read from clamped index");

                    if field.values.is_empty() {
                        // property is not initialized
                        return Ok(AppAction::NoAction);
                    }

                    self.mode = Mode::Graph;
                }

                Ok(AppAction::NoAction)
            }

            Mode::Graph => self.on_key_chart(code),
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect, ctx: MachinesContext, in_focus: bool) {
        let machine = unsafe { &*ctx.selected };

        match self.mode {
            Mode::Navigate => {
                let items = machine.measurements.iter().map(|(_, field)| {
                    let value = match field.values.newest() {
                        Some(sample) => match sample.value {
                            Some(v) => format!("{v:.2}"),
                            None => "null".to_string(),
                        },
                        None => "N/A".to_string(),
                    };

                    (field.label.clone(), value)
                });

                self.navigation.render(frame, area, items, in_focus);
            }
            Mode::Graph => self.render_chart(frame, area, ctx, in_focus),
        }
    }
}
