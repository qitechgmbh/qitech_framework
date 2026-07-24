use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::{self};

use crate::machine::resource::error::RegisterError;

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
