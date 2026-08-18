use chrono::Utc;
use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::symbols;
use ratatui::text::Span;
use ratatui::widgets::Axis;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Chart;
use ratatui::widgets::Dataset;
use ratatui::widgets::GraphType;

use crate::types::KeyResult;
use crate::utils::Timeseries;

pub enum ChartComponentAction {
    NoAction,
    Exit,
}

#[derive(Clone)]
pub struct ChartComponent {
    zoom: u8,
    offset: f64,
}

impl ChartComponent {
    pub fn new() -> Self {
        Self {
            zoom: 0,
            offset: 0.0,
        }
    }

    fn zoom_factor(&self) -> f64 {
        (1u32 << self.zoom as u32) as f64
    }

    fn window(&self) -> f64 {
        600.0 / self.zoom_factor()
    }

    pub fn on_key(&mut self, code: KeyCode) -> KeyResult<ChartComponentAction> {
        match code {
            KeyCode::Left => {
                self.offset += self.window() / 4.0;
                KeyResult::Handled(ChartComponentAction::NoAction)
            }

            KeyCode::Right => {
                self.offset = (self.offset - self.window() / 4.0).max(0.0);
                KeyResult::Handled(ChartComponentAction::NoAction)
            }

            KeyCode::Up | KeyCode::Down => {
                // consume to disable navigation
                KeyResult::Handled(ChartComponentAction::NoAction)
            }

            KeyCode::Esc => KeyResult::Handled(ChartComponentAction::Exit),

            KeyCode::Char('+') => {
                self.zoom = (self.zoom + 1).min(6); // 2^6 = 64x
                KeyResult::Handled(ChartComponentAction::NoAction)
            }

            KeyCode::Char('-') => {
                self.zoom = self.zoom.saturating_sub(1);
                KeyResult::Handled(ChartComponentAction::NoAction)
            }

            _ => KeyResult::Bubble(code),
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, label: &str, timeseries: &Timeseries) {
        let now = timeseries
            .newest()
            .map(|s| s.timestamp)
            .unwrap_or_else(Utc::now);

        let window = self.window();

        // visible age range
        let x_max = -self.offset;
        let x_min = x_max - window;

        let data: Vec<(f64, f64)> = timeseries
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

        let current = timeseries.newest().and_then(|sample| sample.value);

        let title = match current {
            Some(v) => format!(
                "{} ({:.2}) [Zoom {}x | Offset {:.1}s]",
                label,
                v,
                self.zoom_factor(),
                self.offset,
            ),
            None => format!(
                "{} (N/A) [Zoom {}x | Offset {:.1}s]",
                label,
                self.zoom_factor(),
                self.offset,
            ),
        };

        let dataset = Dataset::default()
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Green))
            .data(&data);

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::reset().fg(Color::Yellow));

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
