use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;

use crate::controls::Picker;
use crate::types::AppAction;
use crate::widgets::machines_view::MachinesContext;
use crate::widgets::tab_view::TabItem;

pub struct SubscriptionsView {
    picker: Picker,
    selected: usize,
}

impl TabItem<MachinesContext> for SubscriptionsView {
    fn on_key(&mut self, code: KeyCode, ctx: MachinesContext) -> Result<AppAction, KeyCode> {
        if self.selected == 0 {
            let selected = unsafe { &*ctx.selected };
            let machines = unsafe { &*ctx.machines };

            let candidates: Vec<_> = machines
                .iter()
                .filter(|machine| machine.ident != selected.ident)
                .collect();

            let pick_result = self
                .picker
                .on_key(code, candidates.len().saturating_sub(1))?;

            if let Some(i) = pick_result
                && let Some(machine) = candidates.get(i)
            {
                return Ok(AppAction::Subscribe {
                    provider: machine.ident,
                    subscriber: selected.ident,
                });
            }

            Err(code)
        } else {
            Err(code)
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect, ctx: MachinesContext, in_focus: bool) {
        let selected = unsafe { &*ctx.selected };
        let machines = unsafe { &*ctx.machines };

        let items: Vec<String> = machines
            .iter()
            .filter(|machine| machine.ident != selected.ident)
            .map(|machine| format!("{} ({})", machine.title.as_str(), machine.ident.serial))
            .collect();

        let drop_down_height = self.picker.rendered_height(&items);

        let chunks = Layout::vertical([
            Constraint::Length(drop_down_height as u16),
            Constraint::Min(0),
        ])
        .split(area);

        // Render dropdown
        self.picker.render(frame, chunks[0], in_focus, &items);
    }
}

impl SubscriptionsView {
    pub fn new() -> Self {
        Self {
            selected: 0,
            picker: Picker::new("Select A Machine"),
        }
    }
}
