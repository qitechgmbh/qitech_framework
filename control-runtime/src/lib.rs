use std::time::Duration;
use qitech_lib::ethercat_hal::MasterConfiguration;

// re-export idents as they are common
pub use control_core::MachineIdentification;
pub use control_core::MachineIdentificationUnique;

mod data;
use data::DataStore;
pub use data::DataRegistry;

mod machine_registry;
pub use machine_registry::MachineRegistry;

// mod property;

mod conversion;

pub mod machine;
pub use machine::Machine;
pub use machine::MachineBuild;
pub use machine::BuildContext;
pub use machine::MachineBuildError;
pub use machine::MachineActResult;
pub use machine::MachineActError;
// pub use machine::ConfigProperty;
// pub use machine::StateProperty;

mod ethercat;

mod runtime;
pub use runtime::Runtime;

include!(concat!(env!("OUT_DIR"), "/with_uom.rs"));
pub(crate) use with_uom;

#[derive(Debug, Clone)]
pub struct Config {
    pub interface_discovery_retry_interval: Duration,
    pub ethercat: Option<MasterConfiguration>,
}

pub enum MachineOperationResult {
    Success,
    Failure {
        reason: String,
        can_recover: bool,
    }
}
