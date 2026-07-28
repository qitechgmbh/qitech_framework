use qitech_framework_common::MachineIdentificationUnique;
use qitech_lib::ethercat_hal::EtherCATThreadChannel;
use thiserror::Error;

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

    pub fn ident_unique(&self) -> MachineIdentificationUnique {
        self.ident
    }
}

// --- errors ---
pub type BuildResult<T> = Result<T, BuildError>;

#[derive(Debug, Error)]
pub enum BuildError {
    // --- machine errors ---
    #[error("machine required a valid ethercat interface")]
    UnexpectedMachineIdentification(),

    // --- hardware errors ---
    #[error("machine required a valid ethercat interface")]
    ExpectedEtherCATInterface,

    #[error("expected hardware at index {index}")]
    ExpectedHardwareAtIndex { index: usize },

    #[error("expected an ethercat device with role {role}")]
    ExpectedEtherCATDeviceWithRole { role: u16 },

    #[error("expected an ethercat device at index {index}")]
    ExpectedEtherCATDeviceAtIndex { index: usize },

    #[error("expected a serial device at index {index}")]
    ExpectedSerialDeviceAtIndex { index: usize },

    #[error("failed to configure hardware {0}")]
    HardwareConfig(#[from] anyhow::Error),

    #[error("device type mismatch at index {index}. Expected: {expected}")]
    DeviceTypeMismatch {
        index: usize,
        expected: &'static str,
    },

    // --- resource errors ---
    #[error(transparent)]
    RegisterError(#[from] RegisterError),
}
