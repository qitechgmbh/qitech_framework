//! Modbus RTU as a standalone async module.
//!
//! [`ModbusRtuManager`] owns all RTU lines and runs as its own task. The runtime talks to it
//! only through a [`ModbusRtuHandle`]: it sends requests and drains [`ModbusEvent`]s each
//! tick, without blocking.

mod mod2;

use std::path::PathBuf;

use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;
use tokio_modbus::ExceptionCode;
use tokio_modbus::ProtocolError;
use tokio_modbus::Request;
use tokio_modbus::Response;
use tokio_modbus::SlaveId;

mod line;
mod manager;
mod port;

pub use port::ModbusRtuLineConfig;

/// Index of a line in its manager. Only handed out by [`ModbusRtuManager::register`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModbusRtuLineId(pub(crate) usize);

/// Identifies a request. Unique per handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(u64);

/// Sent from the runtime to the manager.
#[derive(Debug)]
pub(crate) struct ModbusRequest {
    pub(crate) id: RequestId,
    pub(crate) line: ModbusRtuLineId,
    pub(crate) slave_id: SlaveId,
    pub(crate) request: Request<'static>,
}

/// Sent from the manager to the runtime.
#[derive(Debug)]
pub enum ModbusEvent {
    LineConnected {
        line: ModbusRtuLineId,
        path: PathBuf,
    },
    LineDisconnected {
        line: ModbusRtuLineId,
    },
    /// Every request sent through the handle gets exactly one response.
    Response {
        id: RequestId,
        result: Result<Response, ModbusError>,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum ModbusError {
    #[error("line is not available")]
    LineUnavailable,
    #[error("the line's request queue is full")]
    QueueFull,
    #[error("no response within the line's response timeout")]
    Timeout,
    #[error("slave returned exception {0}")]
    Exception(ExceptionCode),
    #[error(transparent)]
    Transport(std::io::Error),
    #[error(transparent)]
    Protocol(ProtocolError),
}

#[derive(Debug, thiserror::Error)]
pub enum SendError {
    #[error("unknown line {0:?}")]
    UnknownLine(ModbusRtuLineId),
    #[error("slave id {0} is outside 1..=247")]
    InvalidSlave(SlaveId),
    #[error("the manager's request queue is full")]
    QueueFull,
    #[error("the manager task has stopped")]
    ManagerGone,
}

/// The runtime's end of a running [`ModbusRtuManager`]. Never blocks.
pub struct ModbusRtuHandle {
    requests: mpsc::Sender<ModbusRequest>,
    events: mpsc::Receiver<ModbusEvent>,
    line_count: usize,
    next_request_id: u64,
}

impl ModbusRtuHandle {
    /// Queues a request on a line. The result arrives later as [`ModbusEvent::Response`].
    pub fn send(
        &mut self,
        line: ModbusRtuLineId,
        slave_id: SlaveId,
        request: Request<'static>,
    ) -> Result<RequestId, SendError> {
        if line.0 >= self.line_count {
            return Err(SendError::UnknownLine(line));
        }
        if !(1..=247).contains(&slave_id) {
            return Err(SendError::InvalidSlave(slave_id));
        }

        let id = RequestId(self.next_request_id);

        let request = ModbusRequest {
            id,
            line,
            slave_id,
            request,
        };
        self.requests.try_send(request).map_err(|err| match err {
            TrySendError::Full(_) => SendError::QueueFull,
            TrySendError::Closed(_) => SendError::ManagerGone,
        })?;

        self.next_request_id += 1;
        Ok(id)
    }

    /// Takes the next event, if any.
    pub fn try_recv(&mut self) -> Option<ModbusEvent> {
        self.events.try_recv().ok()
    }
}
