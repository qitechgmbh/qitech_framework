use crossbeam::channel::Receiver;
use crossbeam::channel::RecvError;
use crossbeam::channel::SendError;
use crossbeam::channel::Sender;
use crossbeam::channel::TryRecvError;
use crossbeam::channel::bounded;

use crate::link::protocol::HandleMessage;
use crate::link::protocol::RuntimeMessage;
use crate::link::session::handle::ReceiveHello;
use crate::link::session::runtime::SendHello;
use crate::link::transport::HandleTransport as HandleTransportTrait;
use crate::link::transport::RuntimeTransport as RuntimeTransportTrait;
use crate::link::transport::Transport;
use crate::link::transport::TransportError;

pub fn new(capacity: usize) -> (SendHello<RuntimeTransport>, ReceiveHello<HandleTransport>) {
    let (hub_tx, runtime_rx) = bounded(capacity);
    let (runtime_tx, hub_rx) = bounded(capacity);

    let hub = HandleTransport {
        tx: hub_tx,
        rx: hub_rx,
    };

    let runtime = RuntimeTransport {
        tx: runtime_tx,
        rx: runtime_rx,
    };

    (SendHello::new(runtime), ReceiveHello::new(hub))
}

// --- handle ---
pub struct HandleTransport {
    tx: Sender<HandleMessage>,
    rx: Receiver<RuntimeMessage>,
}

impl Transport for HandleTransport {
    type In = RuntimeMessage;
    type Out = HandleMessage;

    fn recv(&mut self) -> Result<Self::In, TransportError> {
        Ok(self.rx.recv()?)
    }

    fn try_recv(&mut self) -> Result<Option<Self::In>, TransportError> {
        match self.rx.try_recv() {
            Ok(msg) => Ok(Some(msg)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(TransportError::Disconnected),
        }
    }

    fn send(&mut self, msg: Self::Out) -> Result<(), TransportError> {
        self.tx.send(msg)?;
        Ok(())
    }
}

impl HandleTransportTrait for HandleTransport {}

// --- runtime ---
pub struct RuntimeTransport {
    tx: Sender<RuntimeMessage>,
    rx: Receiver<HandleMessage>,
}

impl Transport for RuntimeTransport {
    type In = HandleMessage;
    type Out = RuntimeMessage;

    fn recv(&mut self) -> Result<Self::In, TransportError> {
        Ok(self.rx.recv()?)
    }

    fn try_recv(&mut self) -> Result<Option<Self::In>, TransportError> {
        match self.rx.try_recv() {
            Ok(msg) => Ok(Some(msg)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(TransportError::Disconnected),
        }
    }

    fn send(&mut self, msg: Self::Out) -> Result<(), TransportError> {
        self.tx.send(msg)?;
        Ok(())
    }
}

impl RuntimeTransportTrait for RuntimeTransport {}

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
