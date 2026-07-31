use crossbeam::channel::Receiver;
use crossbeam::channel::RecvError;
use crossbeam::channel::SendError;
use crossbeam::channel::Sender;
use crossbeam::channel::TryRecvError;
use crossbeam::channel::bounded;

use crate::link::handle::AwaitHello;
use crate::link::protocol::HandleMessage;
use crate::link::protocol::RuntimeMessage;
use crate::link::transport::Transport;
use crate::link::transport::TransportError;

pub fn new_channel(capacity: usize) -> (AwaitHello<HandleTransport>, RuntimeTransport) {
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

    (AwaitHello::new(hub), runtime)
}

// --- handle ---
pub struct HandleTransport {
    tx: Sender<HandleMessage>,
    rx: Receiver<RuntimeMessage>,
}

impl Transport<RuntimeMessage, HandleMessage> for HandleTransport {
    fn recv(&mut self) -> Result<RuntimeMessage, TransportError> {
        Ok(self.rx.recv()?)
    }

    fn try_recv(&mut self) -> Result<Option<RuntimeMessage>, TransportError> {
        match self.rx.try_recv() {
            Ok(msg) => Ok(Some(msg)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(TransportError::ConnectionClosed),
        }
    }

    fn send(&mut self, msg: HandleMessage) -> Result<(), TransportError> {
        self.tx.send(msg)?;
        Ok(())
    }
}

// --- runtime ---
pub struct RuntimeTransport {
    tx: Sender<RuntimeMessage>,
    rx: Receiver<HandleMessage>,
}

impl Transport<HandleMessage, RuntimeMessage> for RuntimeTransport {
    fn recv(&mut self) -> Result<HandleMessage, TransportError> {
        Ok(self.rx.recv()?)
    }

    fn try_recv(&mut self) -> Result<Option<HandleMessage>, TransportError> {
        match self.rx.try_recv() {
            Ok(msg) => Ok(Some(msg)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(TransportError::ConnectionClosed),
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
        TransportError::ConnectionClosed
    }
}

impl<T> From<SendError<T>> for TransportError {
    fn from(_: SendError<T>) -> Self {
        TransportError::ConnectionClosed
    }
}
