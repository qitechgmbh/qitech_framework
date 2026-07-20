use control_core::MachineIdentificationUnique;

use crate::resource::ResolveError;

// --- act ---
#[derive(Debug)]
pub struct ActError {
    pub kind: ActErrorKind,
    pub recoverable: bool,
}

#[derive(Debug)]
pub enum ActErrorKind {
    HardwareFault { details: String },
    InvariantViolation,
}

// --- react ---
pub struct ReactContext<'a> {
    pub config: MachineConfigPropertyReader<'a>,
    pub state: MachineConfigPropertyReader<'a>,
    pub measurements: MachineMeasurementReader<'a>,
}

#[derive(Debug)]
pub struct ReactError {
    pub kind: ReactErrorKind,
    pub recoverable: bool,
}

#[derive(Debug)]
pub enum ReactErrorKind {
    HardwareFault { details: String },
    InvariantViolation,
    ExpiredHandle,
}

// --- subscribe ---
pub struct SubscribeContext<'a> {
    pub ident: MachineIdentificationUnique,
    pub config: MachineConfigPropertyResolver<'a>,
    pub state: MachineStatePropertyResolver<'a>,
    pub measurements: MachineMeasurementResolver<'a>,
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
