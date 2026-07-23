use qitech_lib::ethercat_hal::EtherCATThreadChannel;
use qitech_framework_common::MachineIdentificationUnique;

use crate::machine::resource::{command, config_property, event, measurement, state_property};

mod conversion;
pub mod error;

pub mod hardware;
use hardware::Hardware;

pub mod resource;

pub trait Build: Sized {
    fn build(ctx: BuildContext<'_>) -> error::BuildResult<Self>;
}

pub struct BuildContext<'a> {
    ident: MachineIdentificationUnique,
    ethercat_interface: Option<EtherCATThreadChannel>,
    hardware: Vec<Hardware>,
    config_properties: &'a mut config_property::Manager,
    state_properties: &'a mut state_property::Manager,
    measurements: &'a mut measurement::Manager,
    commands: &'a mut command::Manager,
    events: &'a mut event::Manager,
}

impl<'a> BuildContext<'a> {
    pub(crate) fn new(
        ident: MachineIdentificationUnique,
        ethercat_interface: Option<EtherCATThreadChannel>,
        hardware: Vec<Hardware>,
        config_properties: &'a mut config_property::Manager,
        state_properties: &'a mut state_property::Manager,
        measurements: &'a mut measurement::Manager,
        commands: &'a mut command::Manager,
        events: &'a mut event::Manager,
    ) -> Self {
        Self {
            ident,
            ethercat_interface,
            hardware,
            config_properties,
            state_properties,
            measurements,
            commands,
            events,
        }
    }

    pub fn identification(&self) -> MachineIdentificationUnique {
        self.ident
    }
}
