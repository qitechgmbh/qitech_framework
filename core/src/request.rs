use core::fmt;

use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

use crate::ScalarValue;
use crate::ident::MachineIdentificationUnique;
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
    WriteMachineDeviceInfo {
        /// machine hardware identification
        machine_ident: MachineIdentificationUnique,

        /// role of the device
        role: u16,

        /// ethercat hardware identification
        subdevice_index: usize,
    },

    SetConfigProperty {
        target: MachineIdentificationUnique,
        path: String,
        value: ScalarValue,
    },

    InvokeMachineCommand {
        target: MachineIdentificationUnique,
        path: String,
    },

    SubscribeMachine {
        provider: MachineIdentificationUnique,
        subscriber: MachineIdentificationUnique,
    },

    UnsubscribeMachine {
        provider: MachineIdentificationUnique,
        subscriber: MachineIdentificationUnique,
    },
}

impl fmt::Display for RuntimeRequestKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeRequestKind::WriteMachineDeviceInfo { .. } => {
                write!(f, "WriteMachineDeviceInfo")
            }
            RuntimeRequestKind::SetConfigProperty { .. } => {
                write!(f, "SetConfigProperty")
            }
            RuntimeRequestKind::InvokeMachineCommand { .. } => {
                write!(f, "InvokeMachineCommand")
            }
            RuntimeRequestKind::SubscribeMachine { .. } => {
                write!(f, "SubscribeMachine")
            }
            RuntimeRequestKind::UnsubscribeMachine { .. } => {
                write!(f, "UnsubscribeMachine")
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
    #[error("write machine device info failed")]
    WriteMachineDeviceInfo(#[from] WriteMachineDeviceInfoError),

    #[error("write config property failed")]
    WriteConfigProperty(#[from] WriteConfigPropertyError),

    #[error("invoke machine command failed")]
    InvokeMachineCommand,

    #[error(transparent)]
    MachineSubscribe(#[from] SubscribeError),

    #[error(transparent)]
    MachineUnsubscribe(#[from] UnsubscribeError),
}

// --- errors ---
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum WriteMachineDeviceInfoError {
    #[error("no EtherCAT controller available")]
    NoEtherCATController,

    #[error(transparent)]
    ReadMachineDeviceInfo(#[from] ReadMachineDeviceInfoError),

    #[error("failed to write machine device info to EEPROM: {0}")]
    WriteMachineDeviceInfoEeprom(String),
}

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ReadMachineDeviceInfoError {
    #[error("failed to check if machine device info file exists")]
    CheckExists,

    #[error("failed to read machine device info file")]
    ReadFile,

    #[error("failed to parse machine device info JSON")]
    ParseJson,

    #[error("root JSON value is not an array")]
    RootNotArray,

    #[error("missing device address")]
    MissingDeviceAddress,
}

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum WriteConfigPropertyError {
    #[error("...")]
    MachineNotFound,

    #[error("...")]
    ResourceNotFound,

    #[error("value type mismatch")]
    WriteError(ConfigPropertyWriteError),
}

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum SubscribeError {
    #[error("provider does not have requested machine")]
    ProviderNotFound,

    #[error("provider does not have requested machine")]
    SubscriberNotFound,

    #[error("provider does not have a subscription for subscriber")]
    DuplicateSubscription,

    #[error("unsupported machine")]
    UnsupportedMachine,

    #[error("too many subscriptions")]
    TooManySubscriptions,

    #[error(transparent)]
    ResourceAccess(#[from] ResourceAccessError),
}

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum UnsubscribeError {
    #[error("provider does not have a subscription for subscriber")]
    SubscriptionNotFound,
}
