use core::fmt::Display;
use core::fmt::Formatter;
use std::fmt;

use qitech_framework_common::MachineIdentificationUnique;
use qitech_lib::ethercat_hal::EtherCATThreadChannel;

use crate::machine::Resources;
use crate::machine::hardware::Hardware;
use crate::machine::resource::error::RegisterError;

mod hardware;
mod resource;

pub struct BuildContext<'a> {
    ident: MachineIdentificationUnique,
    ethercat_interface: Option<EtherCATThreadChannel>,
    resources: &'a mut Resources,
    hardware: Vec<Hardware>,
}

impl<'a> BuildContext<'a> {
    pub(crate) fn new(
        ident: MachineIdentificationUnique,
        ethercat_interface: Option<EtherCATThreadChannel>,
        resources: &'a mut Resources,
        hardware: Vec<Hardware>,
    ) -> Self {
        Self {
            ident,
            ethercat_interface,
            resources,
            hardware,
        }
    }
}

// --- errors ---
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
