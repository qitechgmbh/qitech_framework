use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

pub use crate::session::transport::TransportError;

#[derive(Debug, Error)]
pub enum SessionRecvError {
    #[error(transparent)]
    Transport(#[from] TransportError),

    #[error("unexpected message: {0}")]
    UnexpectedMessage(String),
}

#[derive(Debug, Error)]
pub enum HandshakeError {
    #[error("transport error: {0}")]
    Transport(#[from] TransportError),

    #[error("transport error: {0}")]
    Hello(#[from] HelloMatchError),

    // Protocol-level failures
    #[error("unexpected message: {0}")]
    UnexpectedMessage(String),

    #[error("protocol mismatch")]
    ProtocolMismatch,

    // Runtime validation failures
    #[error("schema rejected: {0}")]
    SchemaRejected(#[from] SchemaSyncError),

    #[error("unsupported machine: {0}")]
    UnsupportedMachine(String),

    // Initialization failures
    #[error("initialization failed: {0}")]
    InitializationFailed(String),
}

#[derive(Error, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HelloMatchError {
    #[error("hello magic mismatch: expected {expected:#x}, received {received:#x}")]
    MagicMismatch { expected: u64, received: u64 },

    #[error("protocol version mismatch: expected {expected}, received {received}")]
    ProtocolVersionMismatch { expected: u64, received: u64 },
}

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum SchemaSyncError {
    #[error("duplicate item")]
    DuplicateItem,

    #[error("unsupported QMS version")]
    UnsupportedQmsVersion,

    #[error("cannot resolve schema revision conflict")]
    CannotResolveRevisionConflict,

    #[error("{0}")]
    Custom(String),
}
