use std::time::Duration;
use qitech_lib::ethercat_hal::MasterConfiguration;

mod data;
use data::DataStore;
pub use data::DataRegistry;

mod machine_registry;
pub use machine_registry::MachineRegistry;

pub mod machine;
pub use machine::Machine;
pub use machine::MachineBuild;
pub use machine::MachineBuilder;
pub use machine::MachineBuildError;
pub use machine::MachineActResult;
pub use machine::MachineActError;
pub use machine::ConfigProperty;
pub use machine::StateProperty;
pub use machine::Measurement;

mod ethercat;

mod runtime;
pub use runtime::Runtime;

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
