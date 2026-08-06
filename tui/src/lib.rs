use std::io;
use std::io::Stdout;
use std::panic;
use std::thread;
use std::time::Duration;

use crossbeam::channel::TryRecvError;
use crossterm::event;
use crossterm::event::DisableMouseCapture;
use crossterm::event::EnableMouseCapture;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::execute;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::report::ConfigPropertyStateChange;
use qitech_framework_core::report::ParameterConstraints;
use qitech_framework_core::report::RuntimeInitEvent;
use qitech_framework_core::report::RuntimeInitStatus;
use qitech_framework_core::report::RuntimeReport;
use qitech_framework_core::report::WriteCapability;
use qitech_framework_core::session::ControllerTransport;
use qitech_framework_core::session::controller::SessionHandshake;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

mod session;
mod types;
mod utils;

mod root;
use root::UIRoot;

mod controls;
mod widgets;

use crate::session::SessionMessage;
use crate::types::AppState;
use crate::types::ConfigFieldState;
use crate::types::MachineEntry;
use crate::types::RuntimeStatus;

pub struct TuiConfiguration {
    cycle_time: Duration,
}

impl TuiConfiguration {
    pub fn new() -> Self {
        Self {
            cycle_time: Duration::from_secs_f64(1.0 / 4.0),
        }
    }

    pub fn refresh_rate(mut self, value: Duration) -> Self {
        self.cycle_time = value;
        self
    }
}

impl Default for TuiConfiguration {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    config: TuiConfiguration,
    state: AppState,
    root: UIRoot,
}

impl Tui {
    pub fn create(config: TuiConfiguration) -> anyhow::Result<Self> {
        let original = panic::take_hook();

        panic::set_hook(Box::new(move |panic_info| {
            let _ = disable_raw_mode();
            let mut stdout = io::stdout();
            let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);

            original(panic_info);
        }));

        enable_raw_mode()?;

        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            config,
            state: AppState::new(),
            root: UIRoot::new(),
        })
    }

    pub fn run<T>(mut self, session: SessionHandshake<T>) -> anyhow::Result<()>
    where
        T: ControllerTransport + Send + 'static,
    {
        let (tx, rx) = crossbeam::channel::bounded(128);
        let (tx_action, rx_action) = crossbeam::channel::bounded(128);

        thread::spawn(move || session::run(session, tx, rx_action));

        self.terminal
            .draw(|frame| self.root.render(frame, self.state.as_ctx()))?;

        loop {
            #[allow(clippy::collapsible_if)]
            if event::poll(self.config.cycle_time)?
                && let Event::Key(key) = event::read()?
            {
                match self.root.on_key(key, self.state.as_ctx()) {
                    Ok(action) => {
                        tx_action.send(action).unwrap();
                    }
                    Err(_) => {
                        if key.code == KeyCode::Char('q') {
                            return Ok(());
                        }
                    }
                }
            }

            match rx.try_recv() {
                Ok(msg) => match msg {
                    SessionMessage::Schemas(schemas) => {
                        self.state.schemas = schemas;
                    }
                    SessionMessage::InitEvent(event) => {
                        self.on_init_event(event);
                    }
                    SessionMessage::Report(report) => {
                        self.on_report(*report);
                    }
                    SessionMessage::Disconnected => {
                        self.state.rt_status = RuntimeStatus::Disconnected;
                    }
                },
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => return Ok(()),
            }

            // --- draw ---
            self.terminal
                .draw(|frame| self.root.render(frame, self.state.as_ctx()))?;
        }
    }

    pub fn on_init_event(&mut self, event: RuntimeInitEvent) {
        self.state.rt_status = RuntimeStatus::Initializing(RuntimeInitStatus::from(&event));

        match event {
            RuntimeInitEvent::EtherCATStateUpdate(status) => {
                self.state.ecat_status = status;
            }

            RuntimeInitEvent::BuiltMachine { ident } => {
                self.state.add_machine(ident);
            }

            _ => {}
        }
    }

    pub fn on_report(&mut self, report: RuntimeReport) {
        self.state.rt_status = RuntimeStatus::Running;

        let timestamp = report.timestamp;
        let report = report.machines;

        for mutation in &report.config_property_write_records {
            let Some(entry) = self.find_machine_mut(mutation.ident) else {
                continue;
            };

            let Some(field) = entry.config.get_mut(&mutation.path) else {
                continue;
            };

            if mutation.result.is_err() {
                continue;
            }

            match &mut field.state {
                ConfigFieldState::NotInitialized => {
                    field.state = ConfigFieldState::Initialized {
                        value: mutation.value.clone(),
                        default: mutation.value.clone(),
                        writeable: WriteCapability::Forbidden {
                            reason: "not_initialized".to_string(),
                        },
                        constraints: ParameterConstraints::None,
                    }
                }
                ConfigFieldState::Initialized { value, .. } => {
                    *value = mutation.value.clone();
                }
            }
        }

        for record in &report.config_property_state_records {
            let Some(entry) = self.find_machine_mut(record.ident) else {
                continue;
            };

            let Some(field) = entry.config.get_mut(&record.path) else {
                continue;
            };

            let ConfigFieldState::Initialized {
                default,
                writeable,
                constraints,
                ..
            } = &mut field.state
            else {
                // TODO: print err or ?
                continue;
            };

            match &record.kind {
                ConfigPropertyStateChange::WriteCapability(c) => {
                    *writeable = c.clone();
                }

                ConfigPropertyStateChange::Constraints(c) => {
                    *constraints = c.clone();
                }

                ConfigPropertyStateChange::DefaultValue(v) => {
                    *default = v.clone();
                }
            }
        }

        for mutation in &report.state_mutations {
            let Some(entry) = self.find_machine_mut(mutation.ident) else {
                continue;
            };

            let Some(item) = entry.state.get_mut(&mutation.path) else {
                continue;
            };

            item.value = Some(mutation.value.clone());
        }

        for measurement in &report.measurements {
            let Some(entry) = self.find_machine_mut(*measurement.ident) else {
                continue;
            };

            let Some(item) = entry.measurements.get_mut(measurement.path) else {
                continue;
            };

            item.values.push(timestamp, *measurement.value);
        }

        for command in &report.command_enabled_mutations {
            let Some(entry) = self.find_machine_mut(command.ident) else {
                continue;
            };

            let Some(item) = entry.commands.get_mut(&command.resource) else {
                continue;
            };

            item.enabled = command.can_execute;
        }
    }

    fn find_machine_mut(
        &mut self,
        ident: MachineIdentificationUnique,
    ) -> Option<&mut MachineEntry> {
        self.state.machines.iter_mut().find(|m| m.ident == ident)
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture,);
    }
}
