use chrono::Utc;
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::symbols;
use ratatui::text::Span;
use ratatui::widgets::Axis;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Cell;
use ratatui::widgets::Chart;
use ratatui::widgets::Dataset;
use ratatui::widgets::GraphType;
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
pub struct MeasurementsView {
    selected: usize,
    mode: Mode,

    // --- graph state ---
    zoom: u8,
    offset: f64,
}

impl TabItem<MachinesContext> for MeasurementsView {
    fn on_key(&mut self, code: KeyCode, ctx: MachinesContext) -> Result<AppAction, KeyCode> {
        match self.mode {
            Mode::Navigate => self.on_key_navigation(code, ctx),
            Mode::Graph => self.on_key_chart(code),
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect, ctx: MachinesContext, in_focus: bool) {
        match self.mode {
            Mode::Navigate => self.render_navigation(frame, area, ctx, in_focus),
            Mode::Graph => self.render_chart(frame, area, ctx, in_focus),
        }
    }
}

// navigation
impl MeasurementsView {
    fn on_key_navigation(
        &mut self,
        code: KeyCode,
        ctx: MachinesContext,
    ) -> Result<AppAction, KeyCode> {
        let machine = unsafe { &*ctx.selected };

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
                if self.selected == machine.measurements.len().saturating_sub(1) {
                    return Err(code);
                }

                self.selected += 1;
            }

            _ => return Err(code),
        }

        Ok(AppAction::NoAction)
    }

    fn render_navigation(
        &self,
        frame: &mut Frame,
        area: Rect,
        ctx: MachinesContext,
        in_focus: bool,
    ) {
        let machine = unsafe { &*ctx.selected };

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
}

// chart
impl MeasurementsView {
    fn zoom_factor(&self) -> f64 {
        (1u32 << self.zoom as u32) as f64
    }

    fn window(&self) -> f64 {
        600.0 / self.zoom_factor()
    }

    fn on_key_chart(&mut self, code: KeyCode) -> Result<AppAction, KeyCode> {
        match code {
            KeyCode::Left => {
                self.offset += self.window() / 4.0;
                Ok(AppAction::NoAction)
            }

            KeyCode::Right => {
                self.offset = (self.offset - self.window() / 4.0).max(0.0);
                Ok(AppAction::NoAction)
            }

            KeyCode::Up | KeyCode::Down => {
                // consume to disable navigation
                Ok(AppAction::NoAction)
            }

            KeyCode::Esc => {
                self.mode = Mode::Navigate;
                Ok(AppAction::NoAction)
            }

            KeyCode::Char(' ') => {
                self.mode = Mode::Navigate;
                Ok(AppAction::NoAction)
            }

            KeyCode::Char('+') => {
                self.zoom = (self.zoom + 1).min(6); // 2^6 = 64x
                Ok(AppAction::NoAction)
            }

            KeyCode::Char('-') => {
                self.zoom = self.zoom.saturating_sub(1);
                Ok(AppAction::NoAction)
            }

            _ => Err(code),
        }
    }

    fn render_chart(&self, frame: &mut Frame, area: Rect, ctx: MachinesContext, in_focus: bool) {
        let machine = unsafe { &*ctx.selected };

        let Some((name, field)) = machine.measurements.get_index(self.selected) else {
            return;
        };

        let now = field
            .values
            .newest()
            .map(|s| s.timestamp)
            .unwrap_or_else(Utc::now);

        let window = self.window();

        // Visible age range
        let x_max = -self.offset;
        let x_min = x_max - window;

        let data: Vec<(f64, f64)> = field
            .values
            .iter()
            .filter_map(|sample| {
                sample.value.map(|v| {
                    let age = -((now - sample.timestamp).num_milliseconds() as f64 / 1000.0);

                    (age, v)
                })
            })
            .filter(|(age, _)| *age >= x_min && *age <= x_max)
            .collect();

        let (y_min, y_max) = if data.is_empty() {
            (0.0, 1.0)
        } else {
            #[rustfmt::skip]
            let y_min = data.iter().map(|(_, y)| *y).fold(f64::INFINITY, f64::min);

            #[rustfmt::skip]
            let x_min = data.iter().map(|(_, y)| *y).fold(f64::NEG_INFINITY, f64::max);

            (y_min, x_min)
        };

        // Add a little padding so the line doesn't touch the border.
        let padding = if (y_max - y_min).abs() < f64::EPSILON {
            1.0
        } else {
            (y_max - y_min) * 0.1
        };

        let current = field.values.newest().and_then(|sample| sample.value);

        let title = match current {
            Some(v) => format!(
                "{} ({:.2}) [Zoom {}x | Offset {:.1}s]",
                name,
                v,
                self.zoom_factor(),
                self.offset,
            ),
            None => format!(
                "{} (N/A) [Zoom {}x | Offset {:.1}s]",
                name,
                self.zoom_factor(),
                self.offset,
            ),
        };

        let dataset = Dataset::default()
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Green))
            .data(&data);

        let border_style = if in_focus {
            Style::default().fg(Color::Blue)
        } else {
            Style::default()
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);

        let chart = Chart::new(vec![dataset])
            .block(block)
            .x_axis(
                Axis::default()
                    .bounds([x_min, x_max])
                    .style(Style::reset().fg(Color::White))
                    .labels(vec![
                        Span::raw(format!("{:.1}s", x_min)),
                        Span::raw(format!("{:.1}s", (x_min + x_max) / 2.0)),
                        Span::raw(format!("{:.1}s", x_max)),
                    ]),
            )
            .y_axis(
                Axis::default()
                    .bounds([y_min - padding, y_max + padding])
                    .style(Style::reset().fg(Color::White))
                    .labels(vec![
                        Span::raw(format!("{:.3}", y_min)),
                        Span::raw(format!("{:.3}", (y_min + y_max) / 2.0)),
                        Span::raw(format!("{:.3}", y_max)),
                    ]),
            );

        frame.render_widget(chart, area);
    }
}
