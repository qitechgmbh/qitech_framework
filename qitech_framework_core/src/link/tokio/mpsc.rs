//! In-process transport. Messages are moved through channels without being serialized.

use tokio::sync::mpsc;

use crate::link::protocol::ControllerMessage;
use crate::link::protocol::RuntimeMessage;
use crate::link::transport::ControllerConnector;
use crate::link::transport::ControllerTransport;
use crate::link::transport::RuntimeListener;
use crate::link::transport::RuntimeTransport;
use crate::link::transport::TransportError;

/// Creates a connected listener and connector pair.
/// `capacity` is the number of messages each direction of a connection can buffer.
pub fn link(capacity: usize) -> (MpscRuntimeListener, MpscControllerConnector) {
    // one pending connection, like a listen backlog of one
    let (incoming_tx, incoming_rx) = mpsc::channel(1);

    (
        MpscRuntimeListener {
            incoming: incoming_rx,
        },
        MpscControllerConnector {
            incoming: incoming_tx,
            capacity,
        },
    )
}

// --- runtime ---
pub struct MpscRuntimeListener {
    incoming: mpsc::Receiver<MpscRuntimeTransport>,
}

impl RuntimeListener for MpscRuntimeListener {
    type Transport = MpscRuntimeTransport;

    /// Returns `Disconnected` once every connector has been dropped.
    fn accept(&mut self) -> Result<Self::Transport, TransportError> {
        self.incoming
            .blocking_recv()
            .ok_or(TransportError::Disconnected)
    }
}

pub struct MpscRuntimeTransport {
    tx: mpsc::Sender<RuntimeMessage>,
    rx: mpsc::Receiver<ControllerMessage>,
}

impl RuntimeTransport for MpscRuntimeTransport {
    fn recv(&mut self) -> Result<ControllerMessage, TransportError> {
        self.rx.blocking_recv().ok_or(TransportError::Disconnected)
    }

    fn send(&mut self, msg: RuntimeMessage) -> Result<(), TransportError> {
        self.tx
            .blocking_send(msg)
            .map_err(|_| TransportError::Disconnected)
    }
}

// --- controller ---
#[derive(Clone)]
pub struct MpscControllerConnector {
    incoming: mpsc::Sender<MpscRuntimeTransport>,
    capacity: usize,
}

impl ControllerConnector for MpscControllerConnector {
    type Transport = MpscControllerTransport;

    /// Returns once the connection is queued, the runtime may accept it later.
    async fn connect(&mut self) -> Result<Self::Transport, TransportError> {
        let (controller_tx, runtime_rx) = mpsc::channel(self.capacity);
        let (runtime_tx, controller_rx) = mpsc::channel(self.capacity);

        self.incoming
            .send(MpscRuntimeTransport {
                tx: runtime_tx,
                rx: runtime_rx,
            })
            .await
            .map_err(|_| TransportError::Disconnected)?;

        Ok(MpscControllerTransport {
            tx: controller_tx,
            rx: controller_rx,
        })
    }
}

pub struct MpscControllerTransport {
    tx: mpsc::Sender<ControllerMessage>,
    rx: mpsc::Receiver<RuntimeMessage>,
}

impl ControllerTransport for MpscControllerTransport {
    async fn recv(&mut self) -> Result<RuntimeMessage, TransportError> {
        self.rx.recv().await.ok_or(TransportError::Disconnected)
    }

    async fn send(&mut self, msg: ControllerMessage) -> Result<(), TransportError> {
        self.tx
            .send(msg)
            .await
            .map_err(|_| TransportError::Disconnected)
    }
}
