use std::collections::HashMap;

use qitech_framework::MachineIdentification;
use qitech_framework::MachineInstanceIdentification;
use qitech_framework_core::report::MachinesReport;
use socketioxide::extract::SocketRef;

use crate::api::legacy::adapter::MachineLegacyDataAdapter;
use crate::api::legacy::socketio::events::SocketIOEvent;
use crate::api::types::MachineInstance;
use crate::api::types::MeasurementInfo;

#[derive(Default, Clone)]
pub struct MachineNamespaceManager {
    registry: HashMap<MachineInstanceIdentification, Entry>,
}

impl MachineNamespaceManager {
    pub fn register(&mut self, ident: MachineInstanceIdentification) {
        if self.registry.contains_key(&ident) {
            return;
        }

        self.registry.insert(
            ident,
            Entry {
                sockets: Default::default(),
                instance: Default::default(),
            },
        );
    }

    pub fn update(
        &mut self,
        report: &MachinesReport,
        adapters: &HashMap<MachineIdentification, MachineLegacyDataAdapter>,
    ) {
        for snapshot in &report.measurement_snapshots {
            let Some(entry) = self.registry.get_mut(&snapshot.machine) else {
                // no machine registered under that uid
                continue;
            };

            let Some(info) = entry.instance.measurements.get_mut(&snapshot.path) else {
                // not defined in schema
                continue;
            };

            *info = Some(MeasurementInfo {
                value: snapshot.value,
            });
        }

        for (ident, entry) in &mut self.registry {
            let Some(adapter) = adapters.get(&ident.machine) else {
                continue;
            };

            tracing::info!("dispatching live values for: {ident}");

            let data = (adapter.init_measurements_event)(&entry.instance);
            let event = SocketIOEvent::new("LiveValuesEvent", data);
            Self::broadcast(&mut entry.sockets, event);
        }
    }

    pub fn add_socket(&mut self, socket: SocketRef) {
        let ident = match machine_namespace_path_to_ident(socket.ns()) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Failed to parse NamespaceId: {}", e);

                if let Err(e) = socket.disconnect() {
                    tracing::error!("Failed to disconnect Socket: {}", e);
                }

                return;
            }
        };

        let Some(entry) = self.registry.get_mut(&ident) else {
            tracing::error!("Failed to add socket {ident}: No such machine");

            if let Err(e) = socket.disconnect() {
                tracing::error!("Failed to dsiconnect Socket: {}", e);
            }

            return;
        };

        tracing::info!("Adding new main namespace socket!");

        // --- store the socket ---
        entry.sockets.push(socket);
    }

    fn broadcast(sockets: &mut Vec<SocketRef>, event: SocketIOEvent) {
        sockets.retain(|socket| {
            match socket.emit("event", &event) {
                Ok(()) => true,

                // should not happen
                Err(socketioxide::SendError::Serialize(err)) => {
                    panic!("Who fucked up the payload?: {err}");
                }

                // channel full or disconnected
                Err(socketioxide::SendError::Socket(_)) => false,
            }
        });
    }
}

#[derive(Default, Clone)]
pub struct Entry {
    sockets: Vec<SocketRef>,
    instance: MachineInstance,
}

pub fn machine_namespace_path_to_ident(s: &str) -> Result<MachineInstanceIdentification, String> {
    if let Some(machine_path) = s.strip_prefix("/machine/") {
        let parts: Vec<&str> = machine_path.split('/').collect();
        if parts.len() == 3 {
            let vendor_id = parts[0]
                .parse::<u16>()
                .map_err(|_| "Invalid vendor id".to_string())?;
            let machine_id = parts[1]
                .parse::<u16>()
                .map_err(|_| "Invalid machine id".to_string())?;
            let serial = parts[2]
                .parse::<u16>()
                .map_err(|_| "Invalid serial id".to_string())?;

            return Ok(MachineInstanceIdentification {
                machine: MachineIdentification {
                    vendor_id,
                    machine_id,
                },
                serial,
            });
        }
    }

    Err(format!("Invalid namespace path: {}", s))
}
