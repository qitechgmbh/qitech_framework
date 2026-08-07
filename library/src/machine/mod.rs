use std::any::Any;

use qitech_framework_core::ident::MachineIdentification;
pub use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::report::error::BuildError;
pub use qitech_framework_core::request::SubscribeError;

pub mod error;
use error::ActResult;

mod build;
pub use build::BuildContext;

mod subscribe;
pub use subscribe::RemoteProperty;
pub use subscribe::SubscribeContext;
pub use subscribe::SubscribeResult;

pub(crate) mod hardware;
pub use hardware::Hardware;

pub trait Machine: Any {
    /// defines the update cycle of a machine
    fn act(&mut self) -> ActResult;

    // /// allows a machine to sync remote resources (from subscriptions)
    fn subscribe(&mut self, ctx: SubscribeContext) -> SubscribeResult {
        _ = ctx;
        Err(SubscribeError::UnsupportedMachine)
    }

    /// called when the machine is notified a subscription is canceled
    fn unsubscribe(&mut self, ident: MachineIdentificationUnique) {
        _ = ident
    }
}

pub trait MachineBuild: Sized {
    fn build(ctx: &mut BuildContext) -> Result<Self, BuildError>;
}

pub trait MachineDescriptor {
    const SCHEMA: &'static str;
    const IDENTIFICATION: MachineIdentification;
}
