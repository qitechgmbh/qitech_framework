pub type RuntimeInitializeResult<T> = Result<T, RuntimeInitializeError>;
pub type EtherCATInitializeResult<T> = Result<T, EtherCATInitializeError>;

pub enum RuntimeInitializeError {
    AssertionFailed(&'static str),
    EtherCATError(EtherCATInitializeError),
}

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
