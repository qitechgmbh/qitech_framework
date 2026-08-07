use std::io;
use std::io::Stdout;
use std::panic;
use std::thread;
use std::time::Duration;

use chrono::Local;
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
use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::report::CommandEvent;
use qitech_framework_core::report::ConfigPropertyEvent;
use qitech_framework_core::report::ConfigPropertyWriteOutcome;
use qitech_framework_core::report::RuntimeEvent;
use qitech_framework_core::report::RuntimeInitEvent;
use qitech_framework_core::report::RuntimeInitStatus;
use qitech_framework_core::report::RuntimeReport;
use qitech_framework_core::report::StatePropertyEvent;
use qitech_framework_core::request::RuntimeRequest;
use qitech_framework_core::request::RuntimeRequestKind;
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
use crate::types::AppAction;
use crate::types::AppState;
use crate::types::ConfigFieldState;
use crate::types::MachineEntry;
use crate::types::RuntimeStatus;
use crate::types::Transaction;

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
        let (tx_req, rx_action) = crossbeam::channel::bounded(128);

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
                        let request = match action {
                            AppAction::NoAction => None,
                            AppAction::SetConfig {
                                machine,
                                resource,
                                value,
                            } => Some(RuntimeRequestKind::SetConfigProperty {
                                target: machine,
                                path: resource,
                                value,
                            }),

                            AppAction::ExecuteCommand { machine, resource } => {
                                Some(RuntimeRequestKind::InvokeMachineCommand {
                                    target: machine,
                                    path: resource,
                                })
                            }

                            AppAction::Subscribe {
                                provider,
                                subscriber,
                            } => Some(RuntimeRequestKind::SubscribeMachine {
                                provider,
                                subscriber,
                            }),

                            AppAction::Unsubscribe {
                                provider,
                                subscriber,
                            } => Some(RuntimeRequestKind::UnsubscribeMachine {
                                provider,
                                subscriber,
                            }),
                        };

                        if let Some(kind) = request {
                            let request_id = self.state.transactions.len() as u64;

                            self.state.transactions.push(Transaction {
                                id: request_id,
                                timestamp: Local::now(),
                                request: kind.clone(),
                                result: Ok(()),
                            });

                            tx_req.send(RuntimeRequest { 
                                request_id, 
                                kind, 
                            }).unwrap();
                        }
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

        for response in report.responses {
            let entry = self
                .state
                .transactions
                .get_mut(response.request_id as usize)
                .expect("should map");

            entry.result = response.result;
        }

        for event in report.events {
            match event {
                RuntimeEvent::AddedMachine { ident } => {
                    _ = ident;
                }

                RuntimeEvent::RemovedMachine { ident } => {
                    _ = ident;
                }

                RuntimeEvent::SubscriptionAdded {
                    provider,
                    subscriber,
                    ..
                } => {
                    let Some(entry) = self
                        .state
                        .machines
                        .iter_mut()
                        .find(|m| m.ident == subscriber)
                    else {
                        continue;
                    };

                    entry.subscriptions.insert(provider);
                }

                RuntimeEvent::SubscriptionRemoved {
                    provider,
                    subscriber,
                } => {
                    let Some(entry) = self
                        .state
                        .machines
                        .iter_mut()
                        .find(|m| m.ident == subscriber)
                    else {
                        continue;
                    };

                    entry.subscriptions.retain(|m| *m != provider);
                }
            }
        }

        // --- machines ---
        let timestamp = report.timestamp;
        let report = report.machines;
        for record in report.config_property_records {
            let Some(entry) = self.find_machine_mut(record.machine) else {
                continue;
            };

            let Some(field) = entry.config.get_mut(&record.path) else {
                continue;
            };

            match record.event {
                ConfigPropertyEvent::Registered {
                    default,
                    capability,
                    constraints,
                } => {
                    field.state = ConfigFieldState::Initialized {
                        value: default.clone(),
                        default: default.clone(),
                        capability,
                        constraints,
                    }
                }

                ConfigPropertyEvent::DefaultChanged { after, .. } => {
                    if let ConfigFieldState::Initialized { default, .. } = &mut field.state {
                        *default = after
                    }
                }

                ConfigPropertyEvent::CapabilityChanged { after, .. } => {
                    if let ConfigFieldState::Initialized { capability, .. } = &mut field.state {
                        *capability = after;
                    }
                }

                ConfigPropertyEvent::ConstraintsChanged { after, .. } => {
                    if let ConfigFieldState::Initialized { constraints, .. } = &mut field.state {
                        *constraints = after
                    }
                }

                ConfigPropertyEvent::Written {
                    value: v, outcome, ..
                } => {
                    if !matches!(outcome, ConfigPropertyWriteOutcome::Changed { .. }) {
                        continue;
                    }

                    if let ConfigFieldState::Initialized { value, .. } = &mut field.state {
                        *value = v;
                    }
                }
            }
        }

        for record in report.state_property_records {
            let Some(entry) = self.find_machine_mut(record.machine) else {
                continue;
            };

            let Some(item) = entry.state.get_mut(&record.path) else {
                continue;
            };

            match record.event {
                StatePropertyEvent::Registered { value } => {
                    item.value = Some(value);
                }

                StatePropertyEvent::ValueChanged { after, .. } => {
                    if item.value.is_none() {
                        continue;
                    }

                    item.value = Some(after);
                }
            }
        }

        for snapshot in report.measurement_snapshots {
            let Some(entry) = self.find_machine_mut(snapshot.machine) else {
                continue;
            };

            let Some(item) = entry.measurements.get_mut(&snapshot.path) else {
                continue;
            };

            item.values.push(timestamp, snapshot.value);
        }

        for record in report.command_records {
            let Some(entry) = self.find_machine_mut(record.machine) else {
                continue;
            };

            let Some(item) = entry.commands.get_mut(&record.path) else {
                continue;
            };

            match record.event {
                CommandEvent::Registered => {}
                CommandEvent::CapabilityChanged { after, .. } => {
                    item.enabled = after;
                }
                CommandEvent::Invoke(result) => {
                    _ = result;
                }
            }
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
