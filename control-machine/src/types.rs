use crate::resource::{
    ConfigPropertyReader, ConfigPropertyResolver, MeasurementReader, MeasurementResolver,
    ResolveError, StatePropertyReader, StatePropertyResolver,
};
use control_core::MachineIdentificationUnique;

// --- act ---
#[derive(Debug)]
pub struct ActError {
    pub kind: ActErrorKind,
    pub recoverable: bool,
    pub explanation: String,
}

#[derive(Debug)]
pub enum ActErrorKind {
    HardwareFault,
    InvariantViolation,
}

// --- react ---
pub struct ReactContext<'a> {
    pub config: ConfigPropertyReader<'a>,
    pub state: StatePropertyReader<'a>,
    pub measurements: MeasurementReader<'a>,
}

#[derive(Debug)]
pub struct ReactError {
    pub kind: ReactErrorKind,
    pub recoverable: bool,
    pub explanation: String,
}

#[derive(Debug)]
pub enum ReactErrorKind {
    HardwareFault,
    InvariantViolation,
    ExpiredHandle,
}

// --- subscribe ---
pub struct SubscribeContext<'a> {
    pub ident: MachineIdentificationUnique,
    pub config: ConfigPropertyResolver<'a>,
    pub state: StatePropertyResolver<'a>,
    pub measurements: MeasurementResolver<'a>,
}

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
