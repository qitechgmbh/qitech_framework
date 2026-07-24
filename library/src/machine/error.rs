use core::fmt;
use core::fmt::Formatter;
use std::fmt::Debug;
use std::fmt::Display;

pub use crate::machine::resource::CommandExecuteError;
use crate::machine::resource::error::RegisterError;

// --- build ---
pub type BuildResult<T> = Result<T, BuildError>;

#[derive(Debug)]
pub enum BuildError {
    // --- hardware errors ---
    ExpectedEtherCATInterface,
    ExpectedHardwareAtIndex {
        index: usize,
    },
    ExpectedEtherCATDeviceWithRole {
        role: u16,
    },
    ExpectedEtherCATDeviceAtIndex {
        index: usize,
    },
    ExpectedSerialDeviceAtIndex {
        index: usize,
    },
    DeviceTypeMismatch {
        index: usize,
        expected: &'static str,
    },
    // --- resource errors ---
    RegisterError(RegisterError),
}

impl From<RegisterError> for BuildError {
    fn from(value: RegisterError) -> Self {
        BuildError::RegisterError(value)
    }
}

impl Display for BuildError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExpectedEtherCATInterface => {
                write!(f, "machine required a valid ethercat interface")
            }
            Self::ExpectedHardwareAtIndex { index } => {
                write!(f, "expected hardware at index {index}")
            }
            Self::ExpectedEtherCATDeviceWithRole { role } => {
                write!(f, "expected an ethercat device with role {role}")
            }
            Self::ExpectedEtherCATDeviceAtIndex { index } => {
                write!(f, "expected an ethercat device at index {index}")
            }
            Self::ExpectedSerialDeviceAtIndex { index } => {
                write!(f, "expected a serial device at index {index}")
            }
            Self::DeviceTypeMismatch { index, expected } => {
                write!(
                    f,
                    "device type mismatch at index {index}. Expected: {expected}"
                )
            }
            _ => todo!(),
        }
    }
}

// --- act ---
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

pub type GrantResult = Result<(), GrantError>;

#[derive(Debug)]
pub enum GrantError {
    Rejected,
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
