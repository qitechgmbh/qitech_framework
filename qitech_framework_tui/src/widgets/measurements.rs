use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::components::ChartComponent;
use crate::components::ChartComponentAction;
use crate::components::Navigation;
use crate::types::AppAction;
use crate::types::KeyResult;
use crate::types::MachineEntry;
use crate::widgets::machines_view::MachinesContext;
use crate::widgets::tab_view::TabItem;

#[derive(Clone)]
enum Mode {
    Navigate(Navigation),
    Chart((Navigation, ChartComponent)),
}

pub struct MeasurementsPage {
    mode: Mode,
}

impl MeasurementsPage {
    pub fn new() -> Self {
        Self {
            mode: Mode::Navigate(Default::default()),
        }
    }
}

impl TabItem<MachinesContext> for MeasurementsPage {
    fn on_key(&mut self, code: KeyCode, ctx: MachinesContext) -> KeyResult<AppAction> {
        let machine = unsafe { &*ctx.selected };
        let prop_count = machine.measurements.len().saturating_sub(1);

        if prop_count == 0 {
            return KeyResult::Bubble(code);
        }

        let (mode, result) = match self.mode.clone() {
            Mode::Navigate(navigation) => Self::on_key_navigate(navigation, code, ctx),
            Mode::Chart((navigation, chart)) => Self::on_key_chart(navigation, chart, code),
        };

        self.mode = mode;
        result
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, in_focus: bool, ctx: MachinesContext) {
        let machine = unsafe { &*ctx.selected };

        match &mut self.mode {
            Mode::Navigate(navigation) => {
                Self::render_navigate(navigation, frame, area, in_focus, machine);
            }

            Mode::Chart((navigation, chart)) => {
                Self::render_chart(navigation, chart, frame, area, machine)
            }
        }
    }
}

// --- navigate ---
impl MeasurementsPage {
    fn on_key_navigate(
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

        // --- ensure the value is clamped before reading ---
        navigation.apply_limit(prop_count);

        // --- retrieve and ensure the property is initialized ---
        let (_, field) = ctx
            .selected()
            .measurements
            .get_index(navigation.pos())
            .expect("failed to read from clamped index");

        match field.values.newest() {
            Some(_) if let KeyCode::Char(' ') = code => (
                Mode::Chart((navigation, ChartComponent::new())),
                KeyResult::Handled(AppAction::NoAction),
            ),

            _ => (Mode::Navigate(navigation), KeyResult::Bubble(code)),
        }
    }

    fn render_navigate(
        navigation: &mut Navigation,
        frame: &mut Frame,
        area: Rect,
        in_focus: bool,
        machine: &MachineEntry,
    ) {
        let items = machine.measurements.iter().map(|(_, field)| {
            let value = match &field.values.newest() {
                Some(sample) => match sample.value {
                    Some(v) => format!("{v:.3}"),
                    None => "null".to_string(),
                },
                None => "N/A".to_string(),
            };

            (field.label.clone(), value)
        });

        navigation.render(frame, area, items, in_focus);
    }
}

// --- graph ---
impl MeasurementsPage {
    fn on_key_chart(
        navigation: Navigation,
        mut chart: ChartComponent,
        code: KeyCode,
    ) -> (Mode, KeyResult<AppAction>) {
        match chart.on_key(code) {
            KeyResult::Bubble(code) => (Mode::Chart((navigation, chart)), KeyResult::Bubble(code)),
            KeyResult::Handled(ChartComponentAction::Exit) => {
                (Mode::Navigate(navigation), KeyResult::Bubble(code))
            }
            KeyResult::Handled(ChartComponentAction::NoAction) => (
                Mode::Chart((navigation, chart)),
                KeyResult::Handled(AppAction::NoAction),
            ),
        }
    }

    fn render_chart(
        navigation: &mut Navigation,
        chart: &mut ChartComponent,
        frame: &mut Frame,
        area: Rect,
        machine: &MachineEntry,
    ) {
        let props = &machine.measurements;
        navigation.apply_limit(props.len().saturating_sub(1));

        let pos = navigation.pos();
        let Some((name, field)) = machine.measurements.get_index(pos) else {
            return;
        };

        chart.render(frame, area, name, &field.values);
    }
}
