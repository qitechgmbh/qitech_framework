use qitech_lib::ethercat_hal::EtherCATThreadChannel;
use qitech_framework_common::MachineIdentificationUnique;

use crate::machine::resource::{command, config_property, event, measurement, state_property};

mod conversion;
pub mod error;

pub mod hardware;
use hardware::Hardware;

pub mod resource;

pub trait MachineBuild: Sized {
    fn build(ctx: BuildContext<'_>) -> error::BuildResult<Self>;
}

pub struct BuildContext<'a> {
    pub(crate) ident: MachineIdentificationUnique,
    pub(crate) ethercat_interface: Option<EtherCATThreadChannel>,
    pub(crate) hardware: Vec<Hardware>,
    pub(crate) config_properties: &'a mut config_property::Manager,
    pub(crate) state_properties: &'a mut state_property::Manager,
    pub(crate) measurements: &'a mut measurement::Manager,
    pub(crate) commands: &'a mut command::Manager,
    pub(crate) events: &'a mut event::Manager,
}

impl<'a> BuildContext<'a> {
    pub fn identification(&self) -> MachineIdentificationUnique {
        self.ident
    }
}
