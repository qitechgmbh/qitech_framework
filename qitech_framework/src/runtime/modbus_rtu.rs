use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

use crossbeam::channel;
use qitech_framework_core::ident::MachineInstanceIdentification;

const SERIAL_BY_PATH: &str = "/dev/serial/by-path";
const WATCH_INTERVAL: Duration = Duration::from_millis(500);

pub fn resolve_serial_by_path(binding: &str) -> Option<String> {
    scan_serial_by_path().remove(binding)
}

/// Reads `/dev/serial/by-path` once and maps every topology path to its device (e.g. `/dev/ttyUSB0`)
fn scan_serial_by_path() -> HashMap<String, String> {
    let Ok(dir) = fs::read_dir(Path::new(SERIAL_BY_PATH)) else {
        return HashMap::new();
    };

    dir.filter_map(|entry| {
        let entry = entry.ok()?;
        let device = fs::canonicalize(entry.path()).ok()?;
        Some((
            entry.file_name().to_string_lossy().to_string(),
            device.to_string_lossy().to_string(),
        ))
    })
    .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchEvent {
    /// topology path now resolves to `device`
    Attached { binding: String, device: String },

    /// topology path no longer resolves (or resolves to a different device, followed by `Attached`)
    Detached { binding: String, device: String },
}

/// Spawns a thread polling `/dev/serial/by-path` for the given bindings.
///
/// The initial state is `known`, so only changes from it are reported.
/// The thread exits once the receiver is dropped.
fn spawn_watcher(
    mut known: HashMap<String, Option<String>>,
    tx: channel::Sender<WatchEvent>,
) -> std::io::Result<thread::JoinHandle<()>> {
    thread::Builder::new()
        .name("modbus-rtu-watcher".into())
        .spawn(move || {
            loop {
                let mut current = scan_serial_by_path();

                for (binding, previous) in known.iter_mut() {
                    let resolved = current.remove(binding.as_str());

                    if *previous == resolved {
                        continue;
                    }

                    if let Some(device) = previous.take() {
                        let event = WatchEvent::Detached {
                            binding: binding.clone(),
                            device,
                        };

                        if tx.send(event).is_err() {
                            return;
                        }
                    }

                    if let Some(device) = resolved {
                        *previous = Some(device.clone());

                        let event = WatchEvent::Attached {
                            binding: binding.clone(),
                            device,
                        };

                        if tx.send(event).is_err() {
                            return;
                        }
                    }
                }

                thread::sleep(WATCH_INTERVAL);
            }
        })
}

struct Binding {
    ident: MachineInstanceIdentification,
    device: Option<String>,
}

/// Tracks the modbus rtu bindings, fed by the watcher thread without blocking the runtime loop
pub struct ModbusManager {
    bindings: HashMap<String, Binding>,
    rx: channel::Receiver<WatchEvent>,
    _watcher: thread::JoinHandle<()>,
}

impl ModbusManager {
    /// `bindings` maps topology path -> (owning machine, device resolved during init)
    pub fn new(
        bindings: HashMap<String, (MachineInstanceIdentification, Option<String>)>,
    ) -> std::io::Result<Self> {
        let known = bindings
            .iter()
            .map(|(binding, (_, device))| (binding.clone(), device.clone()))
            .collect();

        let (tx, rx) = channel::unbounded();
        let watcher = spawn_watcher(known, tx)?;

        let bindings = bindings
            .into_iter()
            .map(|(binding, (ident, device))| (binding, Binding { ident, device }))
            .collect();

        Ok(Self {
            bindings,
            rx,
            _watcher: watcher,
        })
    }

    /// Drains pending watcher events without blocking and calls `f` for each with the owning machine
    pub fn update(&mut self, mut f: impl FnMut(MachineInstanceIdentification, &WatchEvent)) {
        while let Ok(event) = self.rx.try_recv() {
            let (WatchEvent::Attached { binding, .. } | WatchEvent::Detached { binding, .. }) =
                &event;

            let Some(entry) = self.bindings.get_mut(binding) else {
                continue;
            };

            entry.device = match &event {
                WatchEvent::Attached { device, .. } => Some(device.clone()),
                WatchEvent::Detached { .. } => None,
            };

            f(entry.ident, &event);
        }
    }
}
