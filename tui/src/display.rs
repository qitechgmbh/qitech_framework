use ratatui::prelude::*;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Cell;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Row;
use ratatui::widgets::Table;

use crate::app::App;
use crate::app::RuntimeStatus;

impl App {
    pub fn display(&self, frame: &mut Frame) {
        let outer = Block::default()
            .borders(Borders::ALL)
            .title("QiTech Control (Terminal Edition)");

        let inner = outer.inner(frame.area());

        frame.render_widget(outer, frame.area());

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // status bar
                Constraint::Min(0),   // page content
            ])
            .split(inner);

        let status = match self.runtime_status {
            RuntimeStatus::Offline => "🔴 Offline",
            RuntimeStatus::Starting => "🟡 Starting",
            RuntimeStatus::Running => "🟢 Running",
        };

        let status = Paragraph::new(format!(
            "Runtime Status: {}",
            status
        ))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
        );

        frame.render_widget(status, chunks[0]);

        if self.pages.is_empty() {
            let error = Paragraph::new("No Machines detected")
                .alignment(Alignment::Center)
                .block(Block::default());

            let area = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Fill(1),
                    Constraint::Length(1),
                    Constraint::Fill(1),
                ])
                .split(chunks[1])[1];

            frame.render_widget(error, area);
            return;
        }

        // has pages
        // chunks[1]
        let page = &self.pages.values().next().unwrap();

        let page_chunks = Layout::vertical([
            Constraint::Length(2 + page.config.len() as u16),
            Constraint::Length(2 + page.state.len() as u16),
            Constraint::Length(2 + page.measurements.len() as u16),
        ])
        .split(chunks[1]);

        // --- config fields ---
        let rows: Vec<Row> = page
            .config
            .iter()
            .map(|(x, field)| {
                let style = Style::default();
                let value = match field.empty {
                    true => "N/A".to_string(),
                    false => format!("{}", field.value.clone()),
                };

                Row::new(
                    vec![
                        Cell::from(field.label.clone()), 
                        Cell::from(value)],
                ).style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [Constraint::Percentage(60), Constraint::Percentage(40)],
        )
        .block(Block::default().borders(Borders::ALL).title("Config"));

        frame.render_widget(table, page_chunks[0]);

        // --- state ---
        let rows: Vec<Row> = page
            .state
            .iter()
            .map(|(x, field)| {
                let style = Style::default();
                let value = match field.empty {
                    true => "N/A".to_string(),
                    false => format!("{}", field.value.clone()),
                };

                Row::new(
                    vec![
                        Cell::from(field.label.clone()), 
                        Cell::from(value)],
                ).style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [Constraint::Percentage(60), Constraint::Percentage(40)],
        )
        .block(Block::default().borders(Borders::ALL).title("State"));

        frame.render_widget(table, page_chunks[1]);

        // --- measurements ---
        let rows: Vec<Row> = page
            .measurements
            .iter()
            .map(|(x, field)| {
                let style = Style::default();
                let value = match field.empty {
                    true => "N/A".to_string(),
                    false => match field.value {
                        Some(v) => format!("{v}"),
                        None => "null".to_string(),
                    },
                };

                Row::new(
                    vec![
                        Cell::from(field.label.clone()), 
                        Cell::from(value)],
                ).style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [Constraint::Percentage(60), Constraint::Percentage(40)],
        )
        .block(Block::default().borders(Borders::ALL).title("Measurements"));

        frame.render_widget(table, page_chunks[2]);
    }
}
