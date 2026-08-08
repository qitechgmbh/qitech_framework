use core::fmt;
use std::fmt::Debug;
use std::fmt::Display;

pub use qitech_framework_core::report::error::ActError;
pub use qitech_framework_core::report::error::ActErrorImpact;
pub use qitech_framework_core::report::error::ActErrorKind;
pub use qitech_framework_core::report::error::BuildError;
pub use qitech_framework_core::request::MachineSubscribeError;
use thiserror::Error;

pub type CommandExecuteResult = Result<(), String>;
pub type BuildResult<T> = Result<T, BuildError>;
pub type SubscribeResult = Result<(), MachineSubscribeError>;

// --- act ---
pub type ActResult = Result<(), ActError>;

// --- validate ---
#[derive(Debug, Error)]
pub enum ValidateError {
    #[error(transparent)]
    OutOfBounds(#[from] BoundsError),
    #[error("{0}")]
    Custom(String),
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
