pub type ActResult = Result<(), ActError>;

#[derive(Debug)]
pub struct ActError {
    pub kind: ActErrorKind,
    pub recoverable: bool,
}

#[derive(Debug)]
pub enum ActErrorKind {
    HardwareFault(String),
    ValidationFailed(String),
}

pub type ReactResult = Result<(), ActError>;

#[derive(Debug)]
pub struct ReactError {
    pub kind: ReactErrorKind,
    pub recoverable: bool,
}

#[derive(Debug)]
pub enum ReactErrorKind {
    HardwareFault(String),
    ValidationFailed(BoundsError),
    ExpiredHandle,
}

#[derive(Debug)]
pub enum ValidateError {
    OutOfBounds(BoundsError),
    Custom(String),
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

// impl From<ResolveError> for SubscribeError {
//     fn from(value: ResolveError) -> Self {
//         match value {
//             ResolveError::NoSuchProperty => SubscribeError::NoSuchResource,
//             ResolveError::InvalidType => SubscribeError::InvalidResourceType,
//         }
//     }
// }

use std::fmt::Debug;
use std::fmt::Display;

#[derive(Debug)]
pub enum BoundsError {
    I64(BoundsErrorAny<i64>),
    F64(BoundsErrorAny<f64>),
}

#[derive(Debug)]
pub struct BoundsErrorAny<T> {
    pub resource_path: &'static str,
    pub received: T,
    pub min: Option<T>,
    pub max: Option<T>,
}

impl<T> Display for BoundsErrorAny<T>
where
    T: Display + Copy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl Display for BoundsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::I64(err) => Display::fmt(err, f),
            Self::F64(err) => Display::fmt(err, f),
        }
    }
}

impl std::error::Error for BoundsError {}
