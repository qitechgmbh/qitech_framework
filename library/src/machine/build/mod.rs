use std::any::TypeId;

use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_lib::ethercat_hal::EtherCATThreadChannel;

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
    ident: MachineIdentificationUnique,

    /// type id of the machine, used for validating builders that accept <M>
    type_id: TypeId,
    type_name: &'static str,

    ethercat_interface: Option<EtherCATThreadChannel>,
    hardware: Vec<Hardware>,

    journals: &'a mut Journals,
    config_properties: ConfigPropertyRegistryRegisterHandle<'a>,
    state_properties: StatePropertyRegistryRegisterHandle<'a>,
    measurements: MeasurementRegistryRegisterHandle<'a>,
}

impl<'a> BuildContext<'a> {
    pub(crate) fn new(
        ident: MachineIdentificationUnique,
        type_id: TypeId,
        type_name: &'static str,
        ethercat_interface: Option<EtherCATThreadChannel>,
        hardware: Vec<Hardware>,
        journals: &'a mut Journals,
        resources: &'a mut Resources,
    ) -> Self {
        Self {
            ident,
            type_id,
            type_name,
            ethercat_interface,
            hardware,
            journals,
            config_properties: resources.config_properties.register_machine(ident),
            state_properties: resources.state_properties.register_machine(ident),
            measurements: resources.measurements.register_machine(ident),
        }
    }

    pub(crate) fn commit_all(self) {
        self.config_properties.commit();
        self.state_properties.commit();
        self.measurements.commit();
    }

    pub fn ident(&self) -> MachineIdentificationUnique {
        self.ident
    }
}
