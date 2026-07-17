use anyhow::bail;
use control_core::MachineIdentification;
use control_core::MachineIdentificationUnique;

mod to_scalar;
use qitech_lib::ethercat_hal::EtherCATThreadChannel;
use to_scalar::ToScalar;

pub mod hardware;
pub use hardware::MachineHardwareRegistry;
pub use hardware::Hardware;
pub use hardware::IdentifiedEthercat;
pub use hardware::IdentifiedModbus;

mod build;
pub use build::MachineBuild;
pub use build::MachineBuilder;
pub use build::MachineBuildError;

mod config;
pub use config::ConfigProperty;
pub use config::BoundedConfigProperty;
pub use config::BoundsError;

mod state;
pub use state::StateProperty;

mod measurement;
pub use measurement::Measurement;
pub use measurement::MeasurementStatistics;

use crate::MachineRegistry;
use crate::MachineRegistryEntry;
use crate::data;
use crate::data::DataRegistry;

pub type MachineActResult = Result<(), MachineActError>;

pub trait Machine {
    fn act(&mut self) -> MachineActResult;

    fn react(&mut self, registry: &DataRegistry) -> MachineActResult { 
        _ = registry; 
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MachineActError {
    pub message: String,
    pub recoverable: bool,
}
