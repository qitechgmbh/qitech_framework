mod to_scalar;

pub mod hardware;
use control_core::MachineIdentificationUnique;
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

mod command;
pub use command::Command;

use crate::data::DataRegistry;

pub type MachineActResult = Result<(), MachineActError>;

pub trait Machine {
    fn act(&mut self) -> MachineActResult;

    fn react(&mut self, registry: &DataRegistry) -> MachineActResult { 
        _ = registry; 
        Ok(())
    }

    fn attach(&mut self, ctx: AttachmentContext) { _ = ctx }
    fn detach(&mut self, ident: MachineIdentificationUnique) { _ = ident }
}

#[derive(Debug, Clone)]
pub struct MachineActError {
    pub message: String,
    pub recoverable: bool,
}



pub struct AttachmentContext {
    registry: &'a DataRegistry,
    ident: MachineIdentificationUnique,
    config: ConfigHandleBuilder,
    state: ConfigHandleBuilder,
    measurements: ConfigHandleBuilder,
}

pub struct ConfigPropertyHandle<T> {
    generation: u64,
    index: usize,
    _marker: T,
}

impl<T> ConfigPropertyHandle<T> {

}

pub struct ConfigHandleBuilder {

}

impl 