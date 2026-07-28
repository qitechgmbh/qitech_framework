use std::collections::HashMap;
use std::io;
use std::thread;
use std::time::Duration;

use crossterm::event::EnableMouseCapture;
use crossterm::event::Event;
use crossterm::event::{self};
use crossterm::execute;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::enable_raw_mode;
use qitech_framework::runtime::bridge::CrossbeamHelloHandle;
use qitech_framework_common::MachineSchema;
use qitech_framework_common::RuntimeInitEvent;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

mod app;
use app::App;

use crate::app::RuntimeStatus;

mod display;

pub fn run(
    schemas: Vec<&str>,
    handle: CrossbeamHelloHandle,
) -> anyhow::Result<()> {
    let mut schemas_parsed = HashMap::new();
    for yaml_str in schemas {
        let schema = MachineSchema::from_yaml_str(yaml_str)?;
        schemas_parsed.insert(schema.identification, schema);
    }

    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // --- init stage ---
    let mut app = App::new(schemas_parsed);
    terminal.draw(|frame| app.display(frame))?;
    thread::sleep(Duration::from_secs_f64(0.2));

    let mut handle = handle.handle_hello().unwrap();
    terminal.draw(|frame| app.display(frame))?;
    thread::sleep(Duration::from_secs_f64(0.2));

    app.runtime_status = RuntimeStatus::Starting;

    let mut handle = loop {
        let event = match handle.recv() {
            RuntimeInitEvent::Finished(handle) => break handle,
            other => other,
        };

        app.handle_init_event(event);
        terminal.draw(|frame| app.display(frame))?;
        
        thread::sleep(Duration::from_secs_f64(0.2));
    };

    app.runtime_status = RuntimeStatus::Running;

    // --- run stage ---
    while app.running() {
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => app.handle_key(key.code, &mut handle),
                Event::Mouse(mouse) => app.handle_mouse(mouse, &mut handle),
                _ => {}
            }
        }

        while let Some(report) = handle.recv() {
            app.handle_report(report);
        }

        terminal.draw(|frame| app.display(frame))?;
    }

    Ok(())
}
