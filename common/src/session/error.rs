use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

use crate::session::transport::TransportError;

#[derive(Debug, Error)]
pub enum HandshakeError {
    #[error("transport error: {0}")]
    Transport(#[from] TransportError),

    // Protocol-level failures
    #[error("unexpected message: {0}")]
    UnexpectedMessage(String),

    #[error("protocol mismatch")]
    ProtocolMismatch,

    // Runtime validation failures
    #[error("schema rejected: {0}")]
    SchemaRejected(String),

    #[error("unsupported machine: {0}")]
    UnsupportedMachine(String),

    // Initialization failures
    #[error("initialization failed: {0}")]
    InitializationFailed(String),
}

pub enum RuntimeSessionHandshakeError {
    SchemaSync(SchemaSyncError),
}

pub enum SchemaSyncError {
    UnsupportedQmsVersion,
    CannotResolveRevisionConflict,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum HelloMatchError {
    MagicMismatch { expected: u64, received: u64 },
    ProtocolVersionMismatch { expected: u64, received: u64 },
}
