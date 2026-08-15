use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::error::TryRecvError;

use crate::session::AsyncControllerTransport;
use crate::session::controller_async;
use crate::session::protocol::ControllerMessage;
use crate::session::protocol::RuntimeMessage;
use crate::session::runtime;
use crate::session::transport::RuntimeTransport;
use crate::session::transport::TransportError;

pub fn tokio_mpsc(
    capacity: usize,
) -> (
    runtime::SessionHandshake<TokioMpscRuntimeTransport>,
    controller_async::SessionHandshake<TokioMpscControllerTransport>,
) {
    let (controller_tx, runtime_rx) = channel(capacity);
    let (runtime_tx, controller_rx) = channel(capacity);

    let controller = TokioMpscControllerTransport {
        tx: controller_tx,
        rx: controller_rx,
    };

    let runtime = TokioMpscRuntimeTransport {
        tx: runtime_tx,
        rx: runtime_rx,
        blocking: true,
    };

    (
        runtime::SessionHandshake::new(runtime),
        controller_async::SessionHandshake::new(controller),
    )
}

// --- controller ---
pub struct TokioMpscControllerTransport {
    tx: Sender<ControllerMessage>,
    rx: Receiver<RuntimeMessage>,
}

impl AsyncControllerTransport for TokioMpscControllerTransport {
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

// --- runtime ---
pub struct TokioMpscRuntimeTransport {
    tx: Sender<RuntimeMessage>,
    rx: Receiver<ControllerMessage>,
    blocking: bool,
}

impl RuntimeTransport for TokioMpscRuntimeTransport {
    fn set_blocking(&mut self, blocking: bool) -> Result<(), TransportError> {
        self.blocking = blocking;
        Ok(())
    }

    fn recv(&mut self) -> Result<ControllerMessage, TransportError> {
        if self.blocking {
            self.rx.blocking_recv().ok_or(TransportError::Disconnected)
        } else {
            match self.rx.try_recv() {
                Ok(msg) => Ok(msg),
                Err(TryRecvError::Empty) => Err(TransportError::WouldBlock),
                Err(TryRecvError::Disconnected) => Err(TransportError::Disconnected),
            }
        }
    }

    fn send(&mut self, msg: RuntimeMessage) -> Result<(), TransportError> {
        self.tx
            .blocking_send(msg)
            .map_err(|_| TransportError::Disconnected)
    }
}
