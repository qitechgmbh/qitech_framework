use qitech_framework_common::MachineIdentification;
use qitech_framework_common::schema::ParseError;
use qitech_framework_common::sync::SendHelloError;
use qitech_framework_common::sync::SubmitStateError;
use qitech_framework_common::sync::SyncRegistryError;
use thiserror::Error;

pub type RuntimeInitializeResult<T> = Result<T, RuntimeInitializeError>;
pub type EtherCATInitializeResult<T> = Result<T, EtherCATInitializeError>;

// --- bootstrap ---
#[derive(Error, Debug)]
pub enum RuntimeInitializeError {
    #[error("bridge initialization failed: {0}")]
    BridgeInitialize(#[from] BridgeBootstrapError),

    #[error("machine already registered: {0:?}")]
    DuplicateMachine(MachineIdentification),

    #[error("failed to read schema: {0}")]
    CannotReadSchema(#[from] ParseError),

    #[error("initialization assertion failed: {0}")]
    AssertionFailed(&'static str),

    #[error("EtherCAT initialization failed")]
    EtherCATError(EtherCATInitializeError),
}

#[derive(Error, Debug)]
pub enum BridgeBootstrapError {
    #[error("bridge disconnected")]
    BridgeDisconnected,

    #[error("failed to send hello")]
    SendHello(#[from] SendHelloError),

    #[error("failed to sync registry")]
    SyncRegistry(#[from] SyncRegistryError),

    #[error("failed to submit state")]
    SubmitState(#[from] SubmitStateError),
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
}
