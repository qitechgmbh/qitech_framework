use qitech_framework_common::MachineIdentification;
use qitech_framework_common::schema::ParseError;
use thiserror::Error;

use crate::runtime::bridge::BridgeInitializeError;

pub type RuntimeInitializeResult<T> = Result<T, RuntimeInitializeError>;
pub type EtherCATInitializeResult<T> = Result<T, EtherCATInitializeError>;

#[derive(Error, Debug)]
pub enum RuntimeInitializeError {
    #[error("bridge initialization failed: {0}")]
    BridgeInitialize(#[from] BridgeInitializeError),

    #[error("machine already registered: {0:?}")]
    DuplicateMachine(MachineIdentification),

    #[error("failed to read schema: {0}")]
    CannotReadSchema(#[from] ParseError),

    #[error("initialization assertion failed: {0}")]
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

/*
// --- impl ---
impl fmt::Display for RuntimeInitializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeInitializeError::DuplicateMachine(machine) => {
                write!(f, "duplicate machine registered: {machine}")
            }
            RuntimeInitializeError::CannotReadSchema(err) => {
                write!(f, "failed to parse machine schema: {err}")
            }
            RuntimeInitializeError::AssertionFailed(msg) => {
                write!(f, "runtime assertion failed: {msg}")
            }
            RuntimeInitializeError::EtherCATError(err) => {
                write!(f, "EtherCAT initialization failed: {err}")
            }
        }
    }
}

impl Error for RuntimeInitializeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RuntimeInitializeError::CannotReadSchema(err) => Some(err),
            RuntimeInitializeError::EtherCATError(err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for EtherCATInitializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EtherCATInitializeError::FailedToSetBeckhoffEepromLockActive(err) => {
                write!(f, "failed to activate Beckhoff EEPROM lock: {err}")
            }
            EtherCATInitializeError::NoResponseFromStateMachineOrTimeout => {
                write!(f, "no response from EtherCAT state machine or timed out")
            }
            EtherCATInitializeError::FailedToRequestStateChange(err) => {
                write!(f, "failed to request EtherCAT state change: {err}")
            }
            EtherCATInitializeError::FailedToGetSubDevices(err) => {
                write!(f, "failed to enumerate EtherCAT subdevices: {err}")
            }
            EtherCATInitializeError::FailedToReachOpState => {
                write!(f, "failed to reach EtherCAT OP state")
            }
        }
    }
}

impl Error for EtherCATInitializeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            EtherCATInitializeError::FailedToSetBeckhoffEepromLockActive(err)
            | EtherCATInitializeError::FailedToRequestStateChange(err)
            | EtherCATInitializeError::FailedToGetSubDevices(err) => Some(err.as_ref()),
            EtherCATInitializeError::NoResponseFromStateMachineOrTimeout
            | EtherCATInitializeError::FailedToReachOpState => None,
        }
    }
}
*/
