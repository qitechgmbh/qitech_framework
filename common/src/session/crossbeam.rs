use crossbeam::channel::Receiver;
use crossbeam::channel::RecvError;
use crossbeam::channel::SendError;
use crossbeam::channel::Sender;
use crossbeam::channel::TryRecvError;
use crossbeam::channel::bounded;

use crate::session::controller;
use crate::session::protocol::ControllerMessage;
use crate::session::protocol::RuntimeMessage;
use crate::session::runtime;
use crate::session::transport::ControllerTransport;
use crate::session::transport::RuntimeTransport;
use crate::session::transport::TransportError;

pub fn crossbeam(
    capacity: usize,
) -> (
    runtime::SessionHandshake<CrossbeamRuntimeTransport>,
    controller::SessionHandshake<CrossbeamControllerTransport>,
) {
    let (hub_tx, runtime_rx) = bounded(capacity);
    let (runtime_tx, hub_rx) = bounded(capacity);

    let hub = CrossbeamControllerTransport {
        tx: hub_tx,
        rx: hub_rx,
    };

    let runtime = CrossbeamRuntimeTransport {
        tx: runtime_tx,
        rx: runtime_rx,
        blocking: true,
    };

    (
        runtime::SessionHandshake::new(runtime),
        controller::SessionHandshake::new(hub),
    )
}

// --- controller ---
pub struct CrossbeamControllerTransport {
    tx: Sender<ControllerMessage>,
    rx: Receiver<RuntimeMessage>,
}

impl ControllerTransport for CrossbeamControllerTransport {
    fn recv(&mut self) -> Result<RuntimeMessage, TransportError> {
        Ok(self.rx.recv()?)
    }

    fn send(&mut self, msg: ControllerMessage) -> Result<(), TransportError> {
        self.tx.send(msg)?;
        Ok(())
    }
}

// --- runtime ---
pub struct CrossbeamRuntimeTransport {
    tx: Sender<RuntimeMessage>,
    rx: Receiver<ControllerMessage>,
    blocking: bool,
}

impl RuntimeTransport for CrossbeamRuntimeTransport {
    fn set_blocking(&mut self, blocking: bool) -> Result<(), TransportError> {
        self.blocking = blocking;
        Ok(())
    }

    fn recv(&mut self) -> Result<ControllerMessage, TransportError> {
        if self.blocking {
            Ok(self.rx.recv()?)
        } else {
            match self.rx.try_recv() {
                Ok(msg) => Ok(msg),

                Err(TryRecvError::Empty) => Err(TransportError::WouldBlock),

                Err(TryRecvError::Disconnected) => Err(TransportError::Disconnected),
            }
        }
    }

    fn send(&mut self, msg: RuntimeMessage) -> Result<(), TransportError> {
        self.tx.send(msg)?;
        Ok(())
    }
}

// --- error conversion ---
impl From<RecvError> for TransportError {
    fn from(_: RecvError) -> Self {
        TransportError::Disconnected
    }
}

impl<T> From<SendError<T>> for TransportError {
    fn from(_: SendError<T>) -> Self {
        TransportError::Disconnected
    }
}
