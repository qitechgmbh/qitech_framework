use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::error::TryRecvError;

use crate::session::ControllerSessionProvider;
use crate::session::ControllerTransport;
use crate::session::RuntimeSessionProvider;
use crate::session::controller;
use crate::session::protocol::ControllerMessage;
use crate::session::protocol::RuntimeMessage;
use crate::session::runtime;
use crate::session::transport::RuntimeTransport;
use crate::session::transport::TransportError;

pub fn mpsc(capacity: usize) -> (MpscRuntimeSessionProvider, MpscControllerSessionProvider) {
    let (controller_tx, runtime_rx) = channel(capacity);
    let (runtime_tx, controller_rx) = channel(capacity);

    let controller = MpscControllerTransport {
        tx: controller_tx,
        rx: controller_rx,
    };

    let runtime = MpscRuntimeTransport {
        tx: runtime_tx,
        rx: runtime_rx,
        blocking: true,
    };

    (
        MpscRuntimeSessionProvider {
            transport: Some(runtime),
        },
        MpscControllerSessionProvider {
            transport: Some(controller),
        },
    )
}

// --- controller provider ---
pub struct MpscControllerSessionProvider {
    transport: Option<MpscControllerTransport>,
}

impl ControllerSessionProvider for MpscControllerSessionProvider {
    type Transport = MpscControllerTransport;

    fn provide(
        &mut self,
    ) -> impl Future<Output = Result<controller::SessionHandshake<Self::Transport>, TransportError>> + Send
    {
        let transport = self.transport.take();

        std::future::ready(
            transport
                .map(controller::SessionHandshake::new)
                .ok_or(TransportError::Disconnected),
        )
    }
}

// --- controller transport ---
pub struct MpscControllerTransport {
    tx: Sender<ControllerMessage>,
    rx: Receiver<RuntimeMessage>,
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

// --- runtime provider ---
pub struct MpscRuntimeSessionProvider {
    transport: Option<MpscRuntimeTransport>,
}

impl RuntimeSessionProvider for MpscRuntimeSessionProvider {
    type Transport = MpscRuntimeTransport;

    fn provide(&mut self) -> Result<runtime::SessionHandshake<Self::Transport>, TransportError> {
        self.transport
            .take()
            .map(runtime::SessionHandshake::new)
            .ok_or(TransportError::Disconnected)
    }
}

// --- runtime transport ---
pub struct MpscRuntimeTransport {
    tx: Sender<RuntimeMessage>,
    rx: Receiver<ControllerMessage>,
    blocking: bool,
}

impl RuntimeTransport for MpscRuntimeTransport {
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
