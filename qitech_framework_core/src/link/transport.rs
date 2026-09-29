use std::io;

use thiserror::Error;

use crate::link::protocol::ControllerMessage;
use crate::link::protocol::RuntimeMessage;

pub trait RuntimeTransport: Send {
    fn recv(&mut self) -> Result<ControllerMessage, TransportError>;
    fn send(&mut self, msg: RuntimeMessage) -> Result<(), TransportError>;
}

pub trait ControllerTransport: Send {
    fn recv(&mut self) -> impl Future<Output = Result<RuntimeMessage, TransportError>> + Send;

    fn send(
        &mut self,
        msg: ControllerMessage,
    ) -> impl Future<Output = Result<(), TransportError>> + Send;
}

pub trait RuntimeListener: Send {
    type Transport: RuntimeTransport;

    fn accept(&mut self) -> Result<Self::Transport, TransportError>;
}

pub trait ControllerConnector: Send {
    type Transport: ControllerTransport;

    fn connect(&mut self) -> impl Future<Output = Result<Self::Transport, TransportError>> + Send;
}

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("connection closed")]
    Disconnected,

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("malformed message: {0}")]
    MalformedMessage(String),

    #[error("frame of {len} bytes exceeds limit of {max} bytes")]
    FrameTooLarge { len: usize, max: usize },

    #[error("peer synchronization lost")]
    PeerSynchronizationLost,
}
