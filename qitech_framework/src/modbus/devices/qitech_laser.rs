use std::fmt;
use std::time::Duration;
use std::time::Instant;

use qitech_lib::common::get_async_runtime;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio_modbus::ExceptionCode;
use tokio_modbus::ProtocolError;
use tokio_modbus::Request;
use tokio_modbus::Response;
use tokio_modbus::client::Client;
use tokio_modbus::client::Context;

use crate::modbus::ModbusDevice;
use crate::modbus::ModbusRequest;
use crate::modbus::ModbusResponse;
use crate::modbus::ModbusSettings;
use crate::modbus::ModbusType;
use crate::modbus::Parity;
use crate::modbus::SerialDeviceMeta;
use crate::modbus::create_modbus_device_context;

const MEASUREMENTS_ADDRESS: u16 = 0x0E;
const MEASUREMENTS_COUNT: u16 = 3;

/// How long a single request may take before it counts as [`LaserError::RequestTimeout`].
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

/// Attempts to reopen the serial port after a transport failure.
const REOPEN_ATTEMPTS: u32 = 3;

/// Pause between reopen attempts, gives a re-enumerating USB adapter time to come back.
const REOPEN_BACKOFF: Duration = Duration::from_millis(200);

/// Quiet time after a malformed frame so the RTU decoder starts on a clean frame again.
/// Well above the 3.5 character gap at 38400 baud (~1ms).
const PROTOCOL_ERROR_SETTLE: Duration = Duration::from_millis(5);

// An internal envelope that pairs a request with its dedicated reply channel
struct ActorMessage {
    request: ModbusRequest,
    reply_tx: oneshot::Sender<Result<ModbusResponse, LaserError>>,
}

pub struct LaserDevice {
    pub measurement: Option<Measurement>,
    /// When the last valid measurement arrived, `None` until the first one
    pub last_measurement_at: Option<Instant>,
    // The synchronous front-end holds the sender channel to talk to the actor
    tx: mpsc::Sender<ActorMessage>,
    // A oneshot receiver tracking a pending in-flight request
    pending_response: Option<oneshot::Receiver<Result<ModbusResponse, LaserError>>>,
    handle: JoinHandle<()>,
}

impl Drop for LaserDevice {
    fn drop(&mut self) {
        // Aborting the actor drops its `Context`, which closes the serial port.
        tracing::debug!("LaserDevice dropped, stopping modbus actor");
        self.handle.abort();
    }
}

impl ModbusDevice for LaserDevice {
    fn send_next_request(&mut self) -> Result<(), anyhow::Error> {
        // If we are already waiting for a response, don't spam another request
        if self.pending_response.is_some() {
            return Ok(());
        }

        let req = Request::ReadInputRegisters(MEASUREMENTS_ADDRESS, MEASUREMENTS_COUNT);
        let (reply_tx, reply_rx) = oneshot::channel();

        let msg = ActorMessage {
            request: req,
            reply_tx,
        };

        // Try sending to the background actor without blocking the synchronous loop
        match self.tx.try_send(msg) {
            Ok(_) => {
                self.pending_response = Some(reply_rx);
            }
            Err(mpsc::error::TrySendError::Full(_)) => {}
            Err(mpsc::error::TrySendError::Closed(_)) => {}
        }

        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn new(
        path: String,
        slave_id: u8,
        settings: Option<ModbusSettings>,
    ) -> Result<Self, anyhow::Error> {
        let meta = match settings {
            Some(s) => SerialDeviceMeta {
                path,
                device_name: None,
                slave_id,
                baudrate: s.baudrate,
                bits: s.bits,
                stop_bits: s.stop_bits,
                parity: s.parity,
                modbus_type: s.modbus_type,
            },
            None => SerialDeviceMeta {
                path,
                device_name: None,
                slave_id,
                baudrate: 38400,
                bits: 8,
                stop_bits: 1,
                parity: Parity::None,
                modbus_type: ModbusType::Rtu,
            },
        };
        let rt = get_async_runtime();
        let _g = rt.enter();
        // Initialize the Modbus Context initially
        let ctx = create_modbus_device_context(&meta)?;
        // Create an bounded command channel for our actor
        let (tx, rx) = mpsc::channel::<ActorMessage>(1);
        // Spawn the long-running async worker thread immediately
        let handle = rt.spawn(run_modbus_actor(rx, ctx, meta));

        Ok(Self {
            measurement: None,
            last_measurement_at: None,
            tx,
            pending_response: None,
            handle,
        })
    }

    fn handle_response(&mut self) -> Result<(), anyhow::Error> {
        let is_ready = match &mut self.pending_response {
            Some(rx) => match rx.try_recv() {
                Ok(result) => Some(result),                       // Response arrived!
                Err(oneshot::error::TryRecvError::Empty) => None, // Still processing
                Err(oneshot::error::TryRecvError::Closed) => {
                    return Err(anyhow::anyhow!(
                        "Oneshot channel dropped without a response"
                    ));
                }
            },
            None => return Ok(()), // No request currently pending
        };

        if let Some(actor_result) = is_ready {
            // Clear out the slot now that we consumed the response
            self.pending_response = None;

            let response = actor_result?; // Propagate errors thrown by the actor loop

            let words = match response {
                Response::ReadInputRegisters(v) => v,
                rsp => {
                    return Err(anyhow::anyhow!(
                        "Invalid Function Code: {}",
                        rsp.function_code()
                    ));
                }
            };

            self.measurement = Some(parse_measurement(&words)?);
            self.last_measurement_at = Some(Instant::now());
        }

        Ok(())
    }
}

fn parse_measurement(words: &[u16]) -> Result<Measurement, anyhow::Error> {
    Ok(Measurement {
        diameter: *words
            .first()
            .ok_or_else(|| anyhow::anyhow!("Laser returned no input registers"))?,
        x_axis: words.get(1).copied(),
        y_axis: words.get(2).copied(),
    })
}

#[derive(Debug)]
pub enum LaserError {
    ModbusError(tokio_modbus::Error),
    TaskDied,
    ModbusException(ExceptionCode),
    /// The serial transport failed (port gone, read/write error). Triggers a reopen.
    IoErr(tokio_modbus::Error),
    /// A malformed or mismatched frame. Transient, the port stays open.
    Protocol(ProtocolError),
    /// The serial port could not be (re)opened, retried on the next request
    PortUnavailable,
    RequestTimeout,
}

impl LaserError {
    /// Whether the error is limited to one request, so the line itself is still usable
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            LaserError::Protocol(_) | LaserError::ModbusException(_) | LaserError::RequestTimeout
        )
    }
}

impl fmt::Display for LaserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LaserError::ModbusError(e) => write!(f, "Modbus error: {}", e),
            LaserError::TaskDied => write!(f, "Internal driver task died"),
            LaserError::ModbusException(code) => write!(f, "Modbus exception: {:?}", code),
            LaserError::IoErr(e) => write!(f, "Hardware I/O error: {}", e),
            LaserError::Protocol(e) => write!(f, "Modbus protocol error: {}", e),
            LaserError::PortUnavailable => write!(f, "Serial port unavailable"),
            LaserError::RequestTimeout => write!(f, "Request timed out"),
        }
    }
}

impl std::error::Error for LaserError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            // These sub-errors implement std::error::Error, so we return them
            LaserError::ModbusError(e) => Some(e),
            LaserError::ModbusException(e) => Some(e),
            LaserError::IoErr(e) => Some(e),
            LaserError::Protocol(e) => Some(e),

            // These errors do not have an underlying nested error payload
            LaserError::TaskDied => None,
            LaserError::PortUnavailable => None,
            LaserError::RequestTimeout => None,
        }
    }
}

/// The long-running asynchronous worker loop.
///
/// Owns the serial `Context`. A transport failure closes the port before it is reopened,
/// since the port is opened exclusively and a second open while the old one is alive fails
/// with `EBUSY`. Protocol errors (malformed frames) leave the port open.
async fn run_modbus_actor(
    mut rx: mpsc::Receiver<ActorMessage>,
    ctx: Context,
    meta: SerialDeviceMeta,
) {
    let mut ctx = Some(ctx);

    // Loop until the LaserDevice front-end is dropped (closing the mpsc channel)
    while let Some(msg) = rx.recv().await {
        // a previous reopen failed, try again before serving this request
        if ctx.is_none() {
            ctx = reopen(&meta).await;
        }

        let res = match &mut ctx {
            Some(c) => modbus_act_msg(msg.request, c).await,
            None => Err(LaserError::PortUnavailable),
        };

        match &res {
            Err(LaserError::IoErr(err)) => {
                tracing::error!("laser modbus transport error: {err}, reopening port");
                close(&mut ctx).await;
                ctx = reopen(&meta).await;
            }
            Err(LaserError::Protocol(err)) => {
                tracing::warn!("laser modbus protocol error: {err}");
                tokio::time::sleep(PROTOCOL_ERROR_SETTLE).await;
            }
            _ => {}
        }

        let _ = msg.reply_tx.send(res);
    }

    // When the loop ends (LaserDevice dropped), cleanly disconnect the serial resource
    close(&mut ctx).await;
    tracing::debug!("LaserDevice background actor shut down cleanly.");
}

/// Disconnects and drops the context, releasing the serial port
async fn close(ctx: &mut Option<Context>) {
    if let Some(mut c) = ctx.take() {
        let _ = c.disconnect().await;
    }
}

/// Opens the serial port again, retrying a few times with a short backoff
async fn reopen(meta: &SerialDeviceMeta) -> Option<Context> {
    for attempt in 1..=REOPEN_ATTEMPTS {
        match create_modbus_device_context(meta) {
            Ok(ctx) => {
                tracing::info!("laser modbus port {} reopened", meta.path);
                return Some(ctx);
            }
            Err(err) => {
                tracing::warn!(
                    "laser modbus reopen of {} failed (attempt {attempt}/{REOPEN_ATTEMPTS}): {err}",
                    meta.path
                );
                tokio::time::sleep(REOPEN_BACKOFF).await;
            }
        }
    }

    None
}

async fn modbus_act_msg(request: ModbusRequest, ctx: &mut Context) -> Result<Response, LaserError> {
    match tokio::time::timeout(REQUEST_TIMEOUT, ctx.call(request)).await {
        Err(_) => Err(LaserError::RequestTimeout),
        Ok(Err(err @ tokio_modbus::Error::Transport(_))) => Err(LaserError::IoErr(err)),
        Ok(Err(tokio_modbus::Error::Protocol(err))) => Err(LaserError::Protocol(err)),
        Ok(Ok(Err(code))) => Err(LaserError::ModbusException(code)),
        Ok(Ok(Ok(response))) => Ok(response),
    }
}

#[derive(Debug, Clone)]
pub struct Measurement {
    pub diameter: u16,
    pub x_axis: Option<u16>,
    pub y_axis: Option<u16>,
}

#[cfg(test)]
mod tests {
    use tokio::io::AsyncReadExt;
    use tokio::io::AsyncWriteExt;
    use tokio::io::DuplexStream;
    use tokio_modbus::Slave;
    use tokio_modbus::client::rtu;

    use super::*;

    fn crc16(data: &[u8]) -> u16 {
        let mut crc: u16 = 0xFFFF;
        for byte in data {
            crc ^= *byte as u16;
            for _ in 0..8 {
                crc = if crc & 1 != 0 {
                    (crc >> 1) ^ 0xA001
                } else {
                    crc >> 1
                };
            }
        }
        crc
    }

    fn frame(payload: &[u8]) -> Vec<u8> {
        let crc = crc16(payload);
        let mut frame = payload.to_vec();
        frame.extend_from_slice(&crc.to_le_bytes());
        frame
    }

    /// Reads one 8 byte ReadInputRegisters request from the client and answers with `reply`
    async fn answer(device: &mut DuplexStream, reply: &[u8]) {
        let mut request = [0u8; 8];
        device.read_exact(&mut request).await.unwrap();
        assert_eq!(request[1], 0x04, "expected ReadInputRegisters");
        device.write_all(&frame(reply)).await.unwrap();
    }

    async fn request(tx: &mpsc::Sender<ActorMessage>) -> Result<ModbusResponse, LaserError> {
        let (reply_tx, reply_rx) = oneshot::channel();
        let msg = ActorMessage {
            request: Request::ReadInputRegisters(MEASUREMENTS_ADDRESS, MEASUREMENTS_COUNT),
            reply_tx,
        };
        tx.send(msg).await.unwrap();
        reply_rx.await.unwrap()
    }

    /// The malformed frame seen in the field (exception for function 0) must not reopen the
    /// port, the next request on the same connection has to succeed.
    #[test]
    fn protocol_error_keeps_port_open() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let (client, mut device) = tokio::io::duplex(256);
            let ctx = rtu::attach_slave(client, Slave(1));

            // a path that cannot be opened, so any reopen attempt would surface as an error
            let meta = SerialDeviceMeta {
                path: "/nonexistent/laser".into(),
                device_name: None,
                slave_id: 1,
                baudrate: 38400,
                bits: 8,
                stop_bits: 1,
                parity: Parity::None,
                modbus_type: ModbusType::Rtu,
            };

            let (tx, rx) = mpsc::channel(1);
            let actor = tokio::spawn(run_modbus_actor(rx, ctx, meta));

            let device_task = tokio::spawn(async move {
                answer(&mut device, &[0x01, 0x80, 0x03]).await;
                answer(
                    &mut device,
                    &[0x01, 0x04, 0x06, 0x06, 0xD6, 0x06, 0xD1, 0x06, 0xDB],
                )
                .await;
                device
            });

            let first = request(&tx).await;
            assert!(
                matches!(first, Err(LaserError::Protocol(_))),
                "malformed frame should be a protocol error, got {first:?}"
            );
            assert!(first.unwrap_err().is_transient());

            let second = request(&tx).await;
            match second {
                Ok(Response::ReadInputRegisters(words)) => {
                    assert_eq!(words, vec![1750, 1745, 1755]);
                }
                other => panic!("expected registers on the same port, got {other:?}"),
            }

            drop(tx);
            actor.await.unwrap();
            device_task.await.unwrap();
        });
    }
}
