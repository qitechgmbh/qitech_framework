use std::io;

use thiserror::Error;

use crate::session::protocol::ControllerMessage;
use crate::session::protocol::RuntimeMessage;

pub trait RuntimeTransport {
    fn set_blocking(&mut self, blocking: bool) -> Result<(), TransportError>;
    fn recv(&mut self) -> Result<ControllerMessage, TransportError>;
    fn send(&mut self, msg: RuntimeMessage) -> Result<(), TransportError>;
}

pub trait ControllerTransport: Send + Sync {
    fn recv(&mut self) -> impl Future<Output = Result<RuntimeMessage, TransportError>> + Send;

    fn send(
        &mut self,
        msg: ControllerMessage,
    ) -> impl Future<Output = Result<(), TransportError>> + Send;
}

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("connection closed")]
    Disconnected,

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("malformed message: {0}")]
    MalformedMessage(String),

    #[error("peer synchronization lost")]
    PeerSynchronizationLost,

    #[error("would block")]
    WouldBlock,
}
