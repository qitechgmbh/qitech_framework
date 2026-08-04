use core::fmt;
use std::fmt::Debug;
use std::fmt::Display;

use thiserror::Error;

pub use crate::machine::build::BuildError;
pub use crate::machine::build::BuildResult;
use crate::machine::resource::subscription::RegisterSubscriptionError;
pub type CommandExecuteResult = Result<(), String>;

// --- act ---
pub type ActResult = Result<(), ActError>;

#[derive(Debug, Error)]
#[error("{kind}")]
pub struct ActError {
    pub kind: ActErrorKind,
    pub recoverable: bool,
}

#[derive(Debug, Error)]
pub enum ActErrorKind {
    #[error("hardware fault: {0}")]
    HardwareFault(String),
    #[error("validation failed: {0}")]
    ValidationFailed(String),
}

// --- validate ---
#[derive(Debug, Error)]
pub enum ValidateError {
    #[error(transparent)]
    OutOfBounds(#[from] BoundsError),
    #[error("{0}")]
    Custom(String),
}

// --- subscribe ---
pub type SubscribeResult<T> = Result<T, MachineSubscribeError>;

#[derive(Debug, Error)]
pub enum MachineSubscribeError {
    #[error("unsupported machine")]
    UnsupportedMachine,

    #[error("too many subscriptions")]
    TooManySubscriptions,

    #[error(transparent)]
    Register(#[from] RegisterSubscriptionError),
}

// --- bounds ---
#[derive(Debug, Error)]
pub enum BoundsError {
    #[error(transparent)]
    I64(#[from] BoundsErrorAny<i64>),
    #[error(transparent)]
    F64(#[from] BoundsErrorAny<f64>),
}

#[derive(Debug)]
pub struct BoundsErrorAny<T: Debug> {
    pub resource: &'static str,
    pub received: T,
    pub min: Option<T>,
    pub max: Option<T>,
}

impl<T> Display for BoundsErrorAny<T>
where
    T: Debug + Display + Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.min, self.max) {
            (Some(min), Some(max)) => write!(
                f,
                "{}: value {} is outside bounds [{}, {}]",
                self.resource, self.received, min, max
            ),
            (Some(min), None) => write!(
                f,
                "{}: value {} is below minimum {}",
                self.resource, self.received, min
            ),
            (None, Some(max)) => write!(
                f,
                "{}: value {} exceeds maximum {}",
                self.resource, self.received, max
            ),
            (None, None) => write!(
                f,
                "{}: value {} failed bounds validation",
                self.resource, self.received
            ),
        }
    }
}

impl<T> std::error::Error for BoundsErrorAny<T> where T: Display + Debug + Copy {}
