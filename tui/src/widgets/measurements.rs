use chrono::Utc;
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::symbols;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Axis;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Cell;
use ratatui::widgets::Chart;
use ratatui::widgets::Dataset;
use ratatui::widgets::GraphType;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Row;
use ratatui::widgets::Table;

use crate::types::AppAction;
use crate::widgets::machines_view::MachinesContext;
use crate::widgets::tab_view::TabItem;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum Mode {
    #[default]
    Navigate,
    Graph,
}

#[derive(Default)]
pub struct MeasurementsPage {
    mode: Mode,
    selected: usize,
    zoom: u8,
}

impl TabItem<MachinesContext> for MeasurementsPage {
    fn on_key(&mut self, code: KeyCode, ctx: MachinesContext) -> Result<AppAction, KeyCode> {
        let machine = unsafe { &*ctx.machine };

        if self.mode == Mode::Graph {
            return match code {
                KeyCode::Char(' ') => {
                    self.mode = Mode::Navigate;
                    Ok(AppAction::NoAction)
                }

                KeyCode::Char('+') => {
                    // zoom in
                    self.zoom = (self.zoom + 1).min(4); // 2^4 = 16x
                    Ok(AppAction::NoAction)
                }

                KeyCode::Char('-') => {
                    // zoom out
                    self.zoom = self.zoom.saturating_sub(1);
                    Ok(AppAction::NoAction)
                }

                _ => Err(code),
            };
        }

        match code {
            KeyCode::Char(' ') => {
                self.mode = Mode::Graph;
            }

            KeyCode::Up => {
                if self.selected == 0 {
                    return Err(code);
                }

                self.selected -= 1;
            }

            KeyCode::Down => {
                let max = machine.measurements.len().saturating_sub(1);
                self.selected = (self.selected + 1).min(max);
            }

            _ => return Err(code),
        }

        Ok(AppAction::NoAction)
    }

    fn render(&self, frame: &mut Frame, area: Rect, ctx: MachinesContext, in_focus: bool) {
        match self.mode {
            Mode::Navigate => self.render_navigation(frame, area, ctx, in_focus),
            Mode::Graph => self.render_chart(frame, area, ctx, in_focus),
        }
    }
}

impl MeasurementsPage {
    fn zoom_factor(&self) -> f64 {
        (1u32 << self.zoom as u32) as f64
    }

    fn window(&self) -> f64 {
        120.0 / self.zoom_factor()
    }

    fn render_navigation(
        &self,
        frame: &mut Frame,
        area: Rect,
        ctx: MachinesContext,
        in_focus: bool,
    ) {
        let machine = unsafe { &*ctx.machine };

        let rows: Vec<Row> = machine
            .measurements
            .iter()
            .enumerate()
            .map(|(i, (_, field))| {
                let selected = i == self.selected;

                let style = if selected && in_focus {
                    Style::reset().fg(Color::LightBlue)
                } else {
                    Style::reset()
                };

                let value = match field.values.newest() {
                    Some(sample) => match sample.value {
                        Some(v) => format!("{:.2}", v),
                        None => "null".to_string(),
                    },
                    None => "N/A".to_string(),
                };

                Row::new(vec![Cell::from(field.label.clone()), Cell::from(value)]).style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [Constraint::Percentage(60), Constraint::Percentage(40)],
        )
        .style(Style::reset());

        frame.render_widget(table, area);
    }

    fn render_chart(&self, frame: &mut Frame, area: Rect, ctx: MachinesContext, _in_focus: bool) {
        let machine = unsafe { &*ctx.machine };

        let Some((name, field)) = machine.measurements.get_index(self.selected) else {
            return;
        };

        let now = field
            .values
            .newest()
            .map(|s| s.timestamp)
            .unwrap_or_else(Utc::now);

        let window = self.window();

        let data: Vec<(f64, f64)> = field
            .values
            .iter()
            .filter_map(|sample| {
                sample.value.map(|v| {
                    let age = -((now - sample.timestamp).num_milliseconds() as f64 / 1000.0);

                    (age, v)
                })
            })
            .filter(|(age, _)| *age >= -window)
            .collect();

        if data.is_empty() {
            return;
        }

        let y_min = data
            .iter()
            .map(|(_, y)| *y)
            .fold(f64::INFINITY, f64::min);

        let y_max = data
            .iter()
            .map(|(_, y)| *y)
            .fold(f64::NEG_INFINITY, f64::max);

        let current = field.values.newest().and_then(|sample| sample.value);

        let title = match current {
            Some(v) => format!("{} ({:.2}) [Zoom {}x]", name, v, self.zoom_factor()),
            None => format!("{} (N/A) [Zoom {}x]", name, self.zoom_factor()),
        };

        let dataset = Dataset::default()
            .name(title)
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Green))
            .data(&data);

        let chart = Chart::new(vec![dataset])
            .x_axis(
                Axis::default()
                    .title("Age (s)")
                    .bounds([-window, 0.0])
                    .labels(vec![
                        Span::raw(format!("{:.1}s", -window)),
                        Span::raw(format!("{:.1}s", -window / 2.0)),
                        Span::raw("0.0s"),
                    ]),
            )
            .y_axis(
                Axis::default()
                    .title("Value")
                    .bounds([y_min, y_max])
                    .labels(vec![
                        Span::raw(format!("{:.2}", y_min)),
                        Span::raw(format!("{:.2}", (y_min + y_max) / 2.0)),
                        Span::raw(format!("{:.2}", y_max)),
                    ]),
            );

        frame.render_widget(chart, area);
    }
}
