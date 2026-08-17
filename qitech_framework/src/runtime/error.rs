use qitech_framework_core::ident::MachineIdentification;
use qitech_framework_core::schema::ParseError;
use qitech_framework_core::session::error::HandshakeError;
use qitech_framework_core::session::error::TransportError;
use thiserror::Error;

pub type RuntimeInitializeResult<T> = Result<T, RuntimeInitializeError>;
pub type EtherCATInitializeResult<T> = Result<T, EtherCATInitializeError>;

#[derive(Error, Debug)]
pub enum RuntimeInitializeError {
    #[error("failed to create runtime session: {0}")]
    CreateSession(TransportError),

    #[error("bridge initialization failed: {0}")]
    Handshake(#[from] HandshakeError),

    #[error("machine already registered: {0:?}")]
    DuplicateMachine(MachineIdentification),

    #[error("failed to read schema: {0}")]
    CannotReadSchema(#[from] ParseError),

    #[error("assertion failed: {0}")]
    AssertionFailed(&'static str),

    #[error("EtherCAT initialization failed")]
    EtherCATError(EtherCATInitializeError),
}

#[derive(Debug)]
pub enum EtherCATInitializeError {
    FailedToSetBeckhoffEepromLockActive(anyhow::Error),
    NoResponseFromStateMachineOrTimeout,
    FailedToRequestStateChange(anyhow::Error),
    FailedToGetSubDevices(anyhow::Error),
    FailedToReachOpState,
}

impl From<EtherCATInitializeError> for RuntimeInitializeError {
    fn from(err: EtherCATInitializeError) -> Self {
        RuntimeInitializeError::EtherCATError(err)
    }
}

// --- runtime ---
#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("lost connection to bridge")]
    BridgeLost,

    #[error("EtherCAT controller died")]
    EtherCATControllerDied,
}
