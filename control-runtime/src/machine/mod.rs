use anyhow::anyhow;
use control_core::MachineIdentificationUnique;

const CONFIG_REGISTRY_ID: usize = 0;
const STATE_REGISTRY_ID: usize = 1;
const MEASUREMENT_REGISTRY_ID: usize = 2;

pub mod hardware;
pub use hardware::MachineHardwareRegistry;
pub use hardware::Hardware;
pub use hardware::IdentifiedEthercat;
pub use hardware::IdentifiedModbus;

mod build;
pub use build::MachineBuild;
pub use build::BuildContext;
pub use build::MachineBuildError;

mod config;
pub use config::ConfigProperty;
pub use config::ConstrainedConfigProperty;
pub type ConfigReaderHandle<T> = data::property::ReaderHandle<CONFIG_REGISTRY_ID, T>;

mod state;
pub use state::StateProperty;
pub type StateReaderHandle<T> = data::property::ReaderHandle<STATE_REGISTRY_ID, T>;

mod measurement;
pub use measurement::Measurement;
pub use measurement::MeasurementStatistics;
pub type MeasurementReaderHandle<T> = data::measurement::ReaderHandle<MEASUREMENT_REGISTRY_ID, T>;

mod command;
pub use command::Command;

use crate::data;

pub type MachineActResult = Result<(), MachineActError>;

pub trait Machine {
    fn act(&mut self) -> MachineActResult;

    fn react(&mut self, ctx: &ReactContext) -> MachineActResult { 
        _ = ctx; 
        Ok(())
    }

    fn subscribe(&mut self, ctx: &SubscribeContext) -> SubscribeResult {
        _ = ctx;
        Err(SubscribeError::OperationNotSupported)
    }

    fn unsubscribe(&mut self, ident: MachineIdentificationUnique) { _ = ident }
}

#[derive(Debug)]
pub struct MachineActError {
    pub error: anyhow::Error,
    pub recoverable: bool,
}

impl From<anyhow::Error> for MachineActError {
    fn from(error: anyhow::Error) -> Self {
        Self { error, recoverable: true }
    }
}

impl From<data::property::ReadError> for MachineActError {
    fn from(value: data::property::ReadError) -> Self {
        _ = value;
        Self { error: anyhow!("Handle expired"), recoverable: false }
    }
}

impl From<data::measurement::ReadError> for MachineActError {
    fn from(value: data::measurement::ReadError) -> Self {
        _ = value;
        Self { error: anyhow!("Handle expired"), recoverable: false }
    }
}

pub struct SubscribeContext<'a> {
    pub ident: MachineIdentificationUnique,
    pub config: data::property::Resolver<'a, CONFIG_REGISTRY_ID, 512>,
    pub state: data::property::Resolver<'a, STATE_REGISTRY_ID, 512>,
    pub measurements: data::measurement::Resolver<'a, MEASUREMENT_REGISTRY_ID, 512>,
}

pub type SubscribeResult = Result<(), SubscribeError>;

#[derive(Debug)]
pub enum SubscribeError {
    OperationNotSupported,
    UnsupportedMachine,
    TooManySubscriptions,
    NoSuchResource,
    InvalidResourceType,
}

impl From<data::property::ResolveError> for SubscribeError {
    fn from(value: data::property::ResolveError) -> Self {
        use data::property::ResolveError;
        match value {
            ResolveError::NoSuchProperty => SubscribeError::NoSuchResource,
            ResolveError::InvalidType => SubscribeError::InvalidResourceType,
        }
    }
}

impl From<data::measurement::ResolveError> for SubscribeError {
    fn from(value: data::measurement::ResolveError) -> Self {
        use data::measurement::ResolveError;
        match value {
            ResolveError::NoSuchProperty => SubscribeError::NoSuchResource,
            ResolveError::InvalidType => SubscribeError::InvalidResourceType,
        }
    }
}

pub struct ReactContext<'a> {
    pub config: data::property::Reader<'a, CONFIG_REGISTRY_ID, 512>,
    pub state: data::property::Reader<'a, STATE_REGISTRY_ID, 512>,
    pub measurements: data::measurement::Reader<'a, MEASUREMENT_REGISTRY_ID, 512>,
}
