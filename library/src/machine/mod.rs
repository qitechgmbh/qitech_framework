use std::any::Any;
use std::time::Duration;

pub use qitech_framework_core::ident::MachineIdentification;
pub use qitech_framework_core::ident::MachineIdentificationUnique;
pub use qitech_framework_core::report::OperationCapability;
pub use qitech_framework_core::report::error::ActError;
pub use qitech_framework_core::report::error::ActErrorImpact;
pub use qitech_framework_core::report::error::ActErrorKind;
pub use qitech_framework_core::report::error::BuildError;
pub use qitech_framework_core::request::MachineSubscribeError;

pub type BuildResult<T> = Result<T, BuildError>;
pub type ActResult = Result<(), ActError>;
pub type CommandExecuteResult = ActResult;
pub type SubscribeResult = Result<(), MachineSubscribeError>;

mod build;
pub use build::BuildContext;

pub(crate) mod hardware;
pub(crate) use hardware::Hardware;

mod subscribe;
pub use subscribe::RemoteProperty;
pub use subscribe::SubscribeContext;

mod config_property;
pub use config_property::ConfigProperty;
pub(crate) use config_property::ConfigPropertyHandle;

mod state_property;
pub use state_property::StateProperty;

mod measurement;
pub use measurement::Measurement;

mod command;
pub(crate) use command::CommandCanExecuteFn;
pub(crate) use command::CommandHandle;

mod event;
pub use event::EventEmitter;

pub trait Machine: Any {
    /// Defines the update cycle of a machine.
    fn act(&mut self, dt: Duration) -> ActResult;

    /// Allows a machine to create remote properties.
    fn subscribe(&mut self, ctx: &mut SubscribeContext) -> SubscribeResult {
        _ = ctx;
        Err(MachineSubscribeError::UnsupportedMachine)
    }

    /// Called when a machine is notified that a subscription was terminated.
    fn unsubscribe(&mut self, ident: MachineIdentificationUnique) {
        _ = ident;
    }
}

pub trait MachineBuild: Sized {
    /// Builds a machine from the provided build context.
    fn build(ctx: &mut BuildContext) -> BuildResult<Self>;
}

/// Provides static identification and schema information for a machine.
pub trait MachineDescriptor {
    const IDENTIFICATION: MachineIdentification;
    const SCHEMA: &'static str;
}
