use std::path::Path;

use tokio::sync::mpsc;
use tokio_modbus::Request;
use tokio_modbus::Slave;
use tokio_modbus::SlaveId;
use tokio_modbus::client::Client;
use tokio_modbus::client::Context;
use tokio_modbus::client::rtu;
use tokio_modbus::slave::SlaveContext;
use tokio_serial::SerialStream;

use crate::modbus::ModbusError;
use crate::modbus::ModbusEvent;
use crate::modbus::ModbusRtuLineConfig;
use crate::modbus::ModbusRtuLineId;
use crate::modbus::RequestId;

/// Requests a line holds before the manager rejects new ones with [`ModbusError::QueueFull`].
const QUEUE_CAPACITY: usize = 64;

#[derive(Debug)]
pub(crate) struct LineRequest {
    pub(crate) id: RequestId,
    pub(crate) slave_id: SlaveId,
    pub(crate) request: Request<'static>,
}

/// Opens the line's serial port and starts its worker task. Returns the line's request queue.
///
/// The worker sends requests one at a time and reports each response as an event. When the
/// queue is closed it fails the remaining requests and exits. When the port fails it reports
/// the line on `failed` and fails every request from then on.
pub(crate) fn spawn(
    id: ModbusRtuLineId,
    config: &ModbusRtuLineConfig,
    path: &Path,
    events: mpsc::Sender<ModbusEvent>,
    failed: mpsc::Sender<ModbusRtuLineId>,
) -> tokio_serial::Result<mpsc::Sender<LineRequest>> {
    let builder = tokio_serial::new(path.to_string_lossy(), config.baud_rate)
        .data_bits(config.data_bits)
        .parity(config.parity)
        .stop_bits(config.stop_bits);
    let port = SerialStream::open(&builder)?;
    let ctx = rtu::attach(port);
    let (tx, rx) = mpsc::channel(QUEUE_CAPACITY);

    tokio::spawn(run(id, ctx, config.clone(), rx, events, failed));
    Ok(tx)
}

async fn run(
    id: ModbusRtuLineId,
    ctx: Context,
    config: ModbusRtuLineConfig,
    mut requests: mpsc::Receiver<LineRequest>,
    events: mpsc::Sender<ModbusEvent>,
    failed: mpsc::Sender<ModbusRtuLineId>,
) {
    let mut ctx = Some(ctx);

    while let Some(request) = requests.recv().await {
        let result = match &mut ctx {
            Some(c) if !requests.is_closed() => {
                let result = call(c, &config, request.slave_id, request.request).await;
                if let Err(ModbusError::Transport(_)) = &result {
                    ctx = None;
                    _ = failed.send(id).await;
                }
                result
            }
            _ => Err(ModbusError::LineUnavailable),
        };

        let response = ModbusEvent::Response {
            id: request.id,
            result,
        };
        _ = events.send(response).await;
    }
}

async fn call(
    ctx: &mut Context,
    config: &ModbusRtuLineConfig,
    slave_id: SlaveId,
    request: Request<'static>,
) -> Result<tokio_modbus::Response, ModbusError> {
    ctx.set_slave(Slave(slave_id));

    match tokio::time::timeout(config.response_timeout, ctx.call(request)).await {
        Err(_) => Err(ModbusError::Timeout),
        Ok(Err(tokio_modbus::Error::Transport(err))) => Err(ModbusError::Transport(err)),
        Ok(Err(tokio_modbus::Error::Protocol(err))) => Err(ModbusError::Protocol(err)),
        Ok(Ok(Err(code))) => Err(ModbusError::Exception(code)),
        Ok(Ok(Ok(response))) => Ok(response),
    }
}
