use std::io;
use std::io::Stdout;
use std::panic;
use std::thread;
use std::time::Duration;

use chrono::Utc;
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
use qitech_framework::MachineIdentificationUnique;
use qitech_framework::link::session::handle::ReceiveHello;
use qitech_framework_common::RuntimeInitEvent;
use qitech_framework_common::RuntimeReport;
use qitech_framework_common::RuntimeStatus;
use qitech_framework_common::link::HandleTransport;
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
use crate::types::MachineEntry;

pub struct TuiConfiguration {
    cycle_time: Duration,
}

impl TuiConfiguration {
    pub fn new() -> Self {
        Self {
            cycle_time: Duration::from_secs_f64(1.0 / 4.0),
        }
    }

    pub fn cycle_time(mut self, value: Duration) -> Self {
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

    pub fn run<T>(mut self, session: ReceiveHello<T>) -> anyhow::Result<()>
    where
        T: HandleTransport + Send + 'static,
    {
        let (tx, rx) = crossbeam::channel::bounded(128);

        thread::spawn(move || session::run(session, tx));

        self.terminal
            .draw(|frame| self.root.render(frame, self.state.as_ctx()))?;

        loop {
            #[allow(clippy::collapsible_if)]
            if event::poll(Duration::from_millis(50))?
                && let Event::Key(key) = event::read()?
            {
                if self.root.on_key(key, self.state.as_ctx()).is_err() {
                    if key.code == KeyCode::Char('q') {
                        return Ok(());
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
                    SessionMessage::Finished => {}
                    SessionMessage::Running => {}
                    SessionMessage::Report(report) => {
                        self.on_report(*report);
                    }
                    SessionMessage::Disconnected => {}
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
        match event {
            RuntimeInitEvent::EtherCATStateUpdate(status) => {
                self.state.ecat_status = status;
            }
            RuntimeInitEvent::EtherCATFinalizing => {
                self.state.rt_status = RuntimeStatus::FinalizingEtherCAT;
            }
            RuntimeInitEvent::EtherCATDiscoveryStarted => {
                self.state.rt_status = RuntimeStatus::DiscoveringEtherCATInterface;
            }
            RuntimeInitEvent::EtherCATDiscoveryCompleted { .. } => {
                self.state.rt_status = RuntimeStatus::Initialized;
            }
            RuntimeInitEvent::EtherCATInitializationStarted => {
                self.state.rt_status = RuntimeStatus::DiscoveringEtherCATInterface;
            }
            RuntimeInitEvent::EtherCATDeviceInitializationFailed { .. } => {
                self.state.rt_status = RuntimeStatus::DiscoveringEtherCATInterface;
            }
            RuntimeInitEvent::EtherCATDeviceInitializationCompleted { .. } => {
                self.state.rt_status = RuntimeStatus::DiscoveringEtherCATInterface;
            }
            RuntimeInitEvent::BuildingMachines => {
                self.state.rt_status = RuntimeStatus::BuildingMachines;
            }
            RuntimeInitEvent::BuiltMachine { ident } => {
                self.state.add_machine(ident);
            }
            RuntimeInitEvent::FailedToBuildMachine { .. } => {}

            RuntimeInitEvent::ModbusDiscoveryStarted => {
                self.state.rt_status = RuntimeStatus::DiscoveringModbusDevices;
            }

            RuntimeInitEvent::Finished => {
                self.state.rt_status = RuntimeStatus::Initialized;
            }
        }
    }

    pub fn on_report(&mut self, report: RuntimeReport) {
        self.state.rt_status = RuntimeStatus::Running { in_pre_op: false };

        let timestamp = report.timestamp;
        let report = report.machines;

        for mutation in &report.config_mutations {
            let Some(entry) = self.find_machine_mut(mutation.machine) else {
                continue;
            };

            let Some(item) = entry.config.get_mut(&mutation.path) else {
                continue;
            };

            item.value = Some(mutation.value.clone());
        }

        for mutation in &report.state_mutations {
            let Some(entry) = self.find_machine_mut(mutation.machine) else {
                continue;
            };

            let Some(item) = entry.state.get_mut(&mutation.path) else {
                continue;
            };

            item.value = Some(mutation.value.clone());
        }

        for measurement in &report.measurements {
            let Some(entry) = self.find_machine_mut(*measurement.machine) else {
                continue;
            };

            let Some(item) = entry.measurements.get_mut(measurement.path) else {
                continue;
            };

            item.values.push(timestamp, *measurement.value);
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
