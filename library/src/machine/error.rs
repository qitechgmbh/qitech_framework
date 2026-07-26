use core::fmt;
use std::fmt::Debug;
use std::fmt::Display;

use thiserror::Error;

pub use crate::machine::build::BuildError;
pub use crate::machine::build::BuildResult;
pub use crate::machine::resource::CommandExecuteError;

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

// --- sync ---
pub type SyncResult = Result<(), SyncError>;

#[derive(Debug, Error)]
#[error("{kind}")]
pub struct SyncError {
    pub kind: SyncErrorKind,
    pub recoverable: bool,
}

#[derive(Debug, Error)]
pub enum SyncErrorKind {
    #[error("hardware fault: {0}")]
    HardwareFault(String),
    #[error(transparent)]
    ValidationFailed(#[from] BoundsError),
    #[error("subscription handle expired")]
    ExpiredHandle,
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
pub type SubscribeResult = Result<(), SubscribeError>;

#[derive(Debug, Error)]
pub enum SubscribeError {
    #[error("subscription rejected")]
    Rejected,
    #[error("unsupported machine")]
    UnsupportedMachine,
    #[error("too many subscriptions")]
    TooManySubscriptions,
    #[error("no such resource")]
    NoSuchResource,
    #[error("invalid resource type")]
    InvalidResourceType,
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
    pub resource_path: &'static str,
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
                self.resource_path, self.received, min, max
            ),
            (Some(min), None) => write!(
                f,
                "{}: value {} is below minimum {}",
                self.resource_path, self.received, min
            ),
            (None, Some(max)) => write!(
                f,
                "{}: value {} exceeds maximum {}",
                self.resource_path, self.received, max
            ),
            (None, None) => write!(
                f,
                "{}: value {} failed bounds validation",
                self.resource_path, self.received
            ),
        }
    }
}

impl<T> std::error::Error for BoundsErrorAny<T> where T: Display + Debug + Copy {}
