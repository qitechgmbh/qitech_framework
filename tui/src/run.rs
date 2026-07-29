use std::collections::HashMap;
use std::io;
use std::println;
use std::time::Duration;

use crossterm::event::DisableMouseCapture;
use crossterm::event::EnableMouseCapture;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::{self};
use crossterm::execute;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use qitech_framework::runtime::bridge::CrossbeamHelloHandle;
use qitech_framework_common::MachineSchema;
use qitech_framework_common::RuntimeInitEvent;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::App;

pub fn run(schemas: Vec<&str>, handle: CrossbeamHelloHandle) -> anyhow::Result<()> {
    let original = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);

        original(panic_info);
    }));

    let mut schemas_parsed = HashMap::new();
    for yaml_str in schemas {
        let schema = MachineSchema::from_yaml_str(yaml_str)?;
        schemas_parsed.insert(schema.identification, schema);
    }

    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let _guard = TerminalGuard;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // --- init stage ---
    let mut app = App::new(schemas_parsed);
    terminal.draw(|frame| app.render(frame))?;

    let mut handle = handle.handle_hello().unwrap();
    terminal.draw(|frame| app.render(frame))?;

    let mut handle = loop {
        let event = match handle.recv() {
            RuntimeInitEvent::Finished(handle) => break handle,
            other => other,
        };

        app.on_init_event_received(event);
        terminal.draw(|frame| app.render(frame))?;
    };

    app.on_init_complete();
    // app.runtime_status = RuntimeStatus::Running;

    // --- run stage ---
    loop {
        if event::poll(Duration::from_millis(50))?
            && let Event::Key(key) = event::read()?
        {
            if key.code == KeyCode::Char('q') {
                break;
            }

            app.on_key_event(key.code, &mut handle);
        }

        while let Some(report) = handle.recv() {
            app.on_report_received(report);
        }

        terminal.draw(|frame| app.render(frame))?;
    }

    println!("EXITING");
    Ok(())
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();

        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture,);
    }
}
