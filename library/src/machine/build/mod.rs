use std::any::TypeId;

use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_lib::ethercat_hal::EtherCATThreadChannel;
use thiserror::Error;

use crate::machine::hardware::Hardware;
use crate::resource::ConfigPropertyRegistryRegisterHandle;
use crate::resource::Journals;
use crate::resource::MeasurementRegistryRegisterHandle;
use crate::resource::Resources;
use crate::resource::StatePropertyRegistryRegisterHandle;

//mod command;
mod config;
mod hardware;
mod measurements;
mod state_property;
// mod resource;

pub struct BuildContext<'a> {
    pub(crate) ident: MachineIdentificationUnique,

    /// type id of the machine, used for validating builders that accept <M>
    pub(crate) type_id: TypeId,

    pub(crate) ethercat_interface: Option<EtherCATThreadChannel>,
    pub(crate) hardware: Vec<Hardware>,

    pub(crate) journals: &'a mut Journals,
    pub(crate) config_properties: ConfigPropertyRegistryRegisterHandle<'a>,
    pub(crate) state_properties: StatePropertyRegistryRegisterHandle<'a>,
    pub(crate) measurements: MeasurementRegistryRegisterHandle<'a>,
}

impl<'a> BuildContext<'a> {
    pub(crate) fn new(
        ident: MachineIdentificationUnique,
        type_id: TypeId,
        ethercat_interface: Option<EtherCATThreadChannel>,
        hardware: Vec<Hardware>,
        journals: &'a mut Journals,
        resources: &'a mut Resources,
    ) -> Self {
        Self {
            ident,
            type_id,
            ethercat_interface,
            hardware,
            journals,
            config_properties: resources.config_properties.register_machine(ident),
            state_properties: resources.state_properties.register_machine(ident),
            measurements: resources.measurements.register_machine(ident),
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
    UnexpectedMachineIdentification,

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
    #[error("attempted to register resource {0} more than once")]
    DuplicateResource(&'static str),

    #[error("resource expected {0} to be set")]
    MissingRequiredField(&'static str),
}
