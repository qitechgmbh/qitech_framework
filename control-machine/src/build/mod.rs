use qitech_lib::ethercat_hal::EtherCATThreadChannel;
use control_core::MachineIdentificationUnique;
use crate::Hardware;
use crate::resource::{ConfigPropertyManager, MeasurementManager, StatePropertyManager};

mod types;
pub use types::BuildError;
pub type BuildResult<T> = Result<T, BuildError>;

mod hardware;
mod config;
mod state;
mod measurement;

pub trait MachineBuild: Sized {
    fn build(ctx: BuildContext<'_>) -> Result<Self, BuildError>;
}

pub struct BuildContext<'a> {
    ident: MachineIdentificationUnique,
    ethercat_interface: Option<EtherCATThreadChannel>,
    hardware: Vec<Hardware>,
    config_properties: &'a mut ConfigPropertyManager,
    state_properties: &'a mut StatePropertyManager,
    measurements: &'a mut MeasurementManager,
}

impl<'a> BuildContext<'a> {
    pub fn new(
        ident: MachineIdentificationUnique,
        ethercat_interface: Option<EtherCATThreadChannel>,
        hardware: Vec<Hardware>,
        config_properties: &'a mut ConfigPropertyManager,
        state_properties: &'a mut StatePropertyManager,
        measurements: &'a mut MeasurementManager,
    ) -> Self {
        Self {
            ident,
            ethercat_interface,
            hardware,
            config_properties,
            state_properties,
            measurements,
        }
    }

    pub fn identification(&self) -> MachineIdentificationUnique {
        self.ident
    }
}
