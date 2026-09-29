use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use serialport::DataBits;
use serialport::Parity;
use serialport::StopBits;
use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior;

use crate::modbus::ModbusBusEvent;
use crate::modbus::ModbusBusRequest;
use crate::modbus::ModbusDevice;
use crate::modbus::ModbusDeviceResponse;
use crate::modbus::ModbusRequestError;
use crate::modbus::ResponseSource;

const SERIAL_BY_PATH_DIR: &str = "/dev/serial/by-path";

// --- bus ---
pub struct ModbusRtuBus {
    pub(crate) config: ModbusRTUBusConfig,
    devices: BTreeMap<u8, ModbusDevice>,
}

/// The runtime dropped its end of a channel, so the bus has nobody to serve.
struct RuntimeGone;

impl ModbusRtuBus {
    /// Drives the bus until the runtime drops one of the channels.
    pub(crate) async fn run(
        mut self,
        events: mpsc::Sender<ModbusBusEvent>,
        mut requests: mpsc::Receiver<ModbusBusRequest>,
    ) {
        _ = self.serve(&events, &mut requests).await;
    }

    async fn serve(
        &mut self,
        events: &mpsc::Sender<ModbusBusEvent>,
        requests: &mut mpsc::Receiver<ModbusBusRequest>,
    ) -> Result<(), RuntimeGone> {
        let target = self.config.port.target();

        let mut scan = tokio::time::interval(self.config.scan_interval);
        scan.set_missed_tick_behavior(MissedTickBehavior::Delay);

        let mut device_path: Option<PathBuf> = None;

        loop {
            tokio::select! {
                _ = scan.tick() => {
                    let resolved = fs::canonicalize(&target).ok();
                    if resolved == device_path {
                        continue;
                    }
                    device_path = resolved;

                    match &device_path {
                        Some(_path) => {
                            // TODO: open the port and start the poll plan
                            send(events, ModbusBusEvent::Enabled).await?;
                        }
                        None => {
                            // fail pending requests before reporting the bus as disabled,
                            // so machines see the failures while they're still enabled
                            self.fail_queued_requests(events).await?;
                            send(events, ModbusBusEvent::Disabled).await?;
                        }
                    }
                }
                request = requests.recv() => {
                    let request = request.ok_or(RuntimeGone)?;
                    self.accept_request(request, device_path.is_some(), events).await?;
                }
            }
        }
    }

    async fn accept_request(
        &mut self,
        request: ModbusBusRequest,
        enabled: bool,
        events: &mpsc::Sender<ModbusBusEvent>,
    ) -> Result<(), RuntimeGone> {
        let device = self.devices.get_mut(&request.slave_id);

        let error = match device {
            Some(device) if enabled => {
                device.enqueue(request.request);
                return Ok(());
            }
            Some(_) => ModbusRequestError::BusDisabled,
            None => ModbusRequestError::UnknownDevice,
        };

        let response = ModbusDeviceResponse {
            slave_id: request.slave_id,
            source: ResponseSource::Request(request.request.id),
            result: Err(error),
        };

        send(events, ModbusBusEvent::Response(response)).await
    }

    async fn fail_queued_requests(
        &mut self,
        events: &mpsc::Sender<ModbusBusEvent>,
    ) -> Result<(), RuntimeGone> {
        for (&slave_id, device) in &mut self.devices {
            for request in device.queued_requests.drain(..) {
                let response = ModbusDeviceResponse {
                    slave_id,
                    source: ResponseSource::Request(request.id),
                    result: Err(ModbusRequestError::BusDisabled),
                };

                send(events, ModbusBusEvent::Response(response)).await?;
            }
        }

        Ok(())
    }
}

async fn send(
    events: &mpsc::Sender<ModbusBusEvent>,
    event: ModbusBusEvent,
) -> Result<(), RuntimeGone> {
    events.send(event).await.map_err(|_| RuntimeGone)
}

// --- config ---
pub struct ModbusRTUBusConfig {
    port: ModbusRtuPort,
    baud_rate: u32,
    data_bits: DataBits,
    parity: Parity,
    stop_bits: StopBits,
    scan_interval: Duration,
}

impl ModbusRTUBusConfig {
    pub fn new(port: ModbusRtuPort, baud_rate: u32) -> Self {
        Self {
            port,
            baud_rate,
            data_bits: DataBits::Eight,
            parity: Parity::Even,
            stop_bits: StopBits::One,
            scan_interval: Duration::from_secs(1),
        }
    }

    pub fn data_bits(mut self, data_bits: DataBits) -> Self {
        self.data_bits = data_bits;
        self
    }

    pub fn parity(mut self, parity: Parity) -> Self {
        self.parity = parity;
        self
    }

    pub fn stop_bits(mut self, stop_bits: StopBits) -> Self {
        self.stop_bits = stop_bits;
        self
    }
}

#[derive(Debug, Hash)]
pub enum ModbusRtuPort {
    Topology(String),
    Path(String),
}

impl ModbusRtuPort {
    pub fn topology(val: impl Into<String>) -> Self {
        Self::Topology(val.into())
    }

    pub fn path(val: impl Into<String>) -> Self {
        Self::Path(val.into())
    }

    /// Returns the unresolved path of the port, which may be a symlink or not exist yet.
    fn target(&self) -> PathBuf {
        match self {
            Self::Topology(name) => Path::new(SERIAL_BY_PATH_DIR).join(name),
            Self::Path(path) => PathBuf::from(path),
        }
    }
}
