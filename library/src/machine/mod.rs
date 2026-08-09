use std::any::Any;
use std::time::Instant;

pub use qitech_framework_core::ident::MachineIdentification;
pub use qitech_framework_core::ident::MachineIdentificationUnique;
pub use qitech_framework_core::report::OperationCapability;
pub use qitech_framework_core::request::MachineSubscribeError;

pub mod error;
use error::ActResult;
use error::BuildResult;
use error::SubscribeResult;

mod build;
pub use build::BuildContext;

mod subscribe;
pub use subscribe::RemoteProperty;
pub use subscribe::SubscribeContext;

mod config_property;
pub use config_property::ConfigProperty;
pub(crate) use config_property::ConfigPropertyHandle;

mod state_property;
pub use state_property::StateProperty;

pub(crate) mod hardware;
pub(crate) use hardware::Hardware;

mod measurement;
pub use measurement::Measurement;

mod command;
pub(crate) use command::CommandCanExecuteFn;
pub(crate) use command::CommandExecuteFn;
pub(crate) use command::CommandHandle;

pub trait Machine: Any {
    /// defines the update cycle of a machine
    fn act(&mut self, now: Instant) -> ActResult;

    // /// allows a machine to sync remote resources (from subscriptions)
    fn subscribe(&mut self, ctx: &mut SubscribeContext) -> SubscribeResult {
        _ = ctx;
        Err(MachineSubscribeError::UnsupportedMachine)
    }

    /// called when the machine is notified a subscription was terminated
    fn unsubscribe(&mut self, ident: MachineIdentificationUnique) {
        _ = ident
    }
}

pub trait MachineBuild: Sized {
    fn build(ctx: &mut BuildContext) -> BuildResult<Self>;
}

pub trait MachineDescriptor {
    const IDENTIFICATION: MachineIdentification;
    const SCHEMA: &'static str;
}
