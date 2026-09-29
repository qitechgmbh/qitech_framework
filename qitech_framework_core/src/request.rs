use core::fmt;

use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

use crate::ScalarValue;
use crate::ident::MachineInstanceId;
use crate::report::CommandExecuteError;
use crate::report::ConfigPropertyWriteError;
use crate::report::ResourceAccessError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeRequest {
    /// identifier the controller can use to map request back to response
    pub request_id: u64,

    /// the actual request
    pub kind: RuntimeRequestKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeRequestKind {
    SetConfigProperty {
        target: MachineInstanceId,
        path: String,
        value: ScalarValue,
    },

    ExecuteCommand {
        target: MachineInstanceId,
        path: String,
    },

    SubscribeMachine {
        provider: MachineInstanceId,
        subscriber: MachineInstanceId,
    },

    UnsubscribeMachine {
        provider: MachineInstanceId,
        subscriber: MachineInstanceId,
    },

    Terminate,
}

impl fmt::Display for RuntimeRequestKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeRequestKind::SetConfigProperty { .. } => {
                write!(f, "SetConfigProperty")
            }
            RuntimeRequestKind::ExecuteCommand { .. } => {
                write!(f, "InvokeMachineCommand")
            }
            RuntimeRequestKind::SubscribeMachine { .. } => {
                write!(f, "SubscribeMachine")
            }
            RuntimeRequestKind::UnsubscribeMachine { .. } => {
                write!(f, "UnsubscribeMachine")
            }
            RuntimeRequestKind::Terminate => {
                write!(f, "Terminate")
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeResponse {
    pub request_id: u64,
    pub result: Result<(), RuntimeRequestError>,
}

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeRequestError {
    #[error(transparent)]
    MachineSetConfigProperty(#[from] MachineSetConfigProperty),

    #[error(transparent)]
    MachineExecuteCommand(#[from] MachineExecuteCommandError),

    #[error(transparent)]
    MachineSubscribe(#[from] MachineSubscribeError),

    #[error(transparent)]
    MachineUnsubscribe(#[from] MachineUnsubscribeError),
}

// --- errors ---
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum MachineSetConfigProperty {
    #[error(transparent)]
    ResourceAccess(#[from] ResourceAccessError),

    #[error(transparent)]
    WriteError(#[from] ConfigPropertyWriteError),
}

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum MachineExecuteCommandError {
    #[error(transparent)]
    ResourceAccess(#[from] ResourceAccessError),

    #[error(transparent)]
    ExecuteError(#[from] CommandExecuteError),
}

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum MachineSubscribeError {
    #[error(transparent)]
    ResourceAccess(#[from] ResourceAccessError),

    #[error("provider not found")]
    ProviderNotFound,

    #[error("subscriber not found")]
    SubscriberNotFound,

    #[error("subscriber is already subscribed")]
    AlreadySubscribed,

    #[error("machine is not supported")]
    UnsupportedMachine,

    #[error("subscription limit exceeded")]
    TooManySubscriptions,
}

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum MachineUnsubscribeError {
    #[error("subscription not found")]
    SubscriptionNotFound,
}
