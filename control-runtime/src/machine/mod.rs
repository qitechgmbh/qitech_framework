use anyhow::anyhow;
use control_core::MachineIdentificationUnique;
use crate::resource::{
    MachineConfigPropertyAccessHandle, MachineConfigPropertyResolver, 
    MachineMeasurementAccessHandle, MachineMeasurementResolver
};

pub mod hardware;
pub use hardware::MachineHardwareRegistry;
pub use hardware::Hardware;
pub use hardware::IdentifiedEthercat;
pub use hardware::IdentifiedModbus;

mod build;
pub use build::MachineBuild;
pub use build::MachineBuildContext;
pub use build::MachineBuildError;

mod config;
pub use config::ConfigProperty;
pub use config::ConstrainedConfigProperty;
pub type ConfigPropertyAccessHandle<T> = MachineConfigPropertyAccessHandle<T>;

mod state;
pub use state::StateProperty;
pub type StatePropertyAccessHandle<T> = MachineStatePropertyAccessHandle<T>;

mod measurement;
pub use measurement::Measurement;
pub use measurement::MeasurementStatistics;
pub type MeasurementReaderHandle<T> = MachineMeasurementAccessHandle<T>;

mod command;
pub use command::Command;

mod event;
pub use event::EventEmitter;

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

impl From<resource::ReadError> for MachineActError {
    fn from(value: resource::ReadError) -> Self {
        _ = value;
        Self { error: anyhow!("Handle expired"), recoverable: true }
    }
}

pub struct SubscribeContext<'a> {
    pub ident: MachineIdentificationUnique,
    pub config: MachineConfigPropertyResolver<'a>,
    pub state: MachineStatePropertyResolver<'a>,
    pub measurements: MachineMeasurementResolver<'a>,
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

impl From<ResolveError> for SubscribeError {
    fn from(value: ResolveError) -> Self {
        match value {
            ResolveError::NoSuchProperty => SubscribeError::NoSuchResource,
            ResolveError::InvalidType => SubscribeError::InvalidResourceType,
        }
    }
}

pub struct ReactContext<'a> {
    pub config: MachineConfigPropertyReader<'a>,
    pub state: MachineConfigPropertyReader<'a>,
    pub measurements: MachineMeasurementReader<'a>,
}
