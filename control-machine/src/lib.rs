use control_core::MachineIdentificationUnique;

// exposes with_uom!() for macro calls that operate on uom units
// generates the macro from the units.toml file using build.rs
include!(concat!(env!("OUT_DIR"), "/with_uom.rs"));
// pub(crate) use with_uom;

mod types;
pub use types::Hardware;
pub use types::ActError;
pub use types::ActErrorKind;
pub use types::ReactContext;
pub use types::ReactError;
pub use types::ReactErrorKind;
pub use types::SubscribeContext;
pub use types::SubscribeError;

pub type ActResult = Result<(), ActError>;
pub type ReactResult = Result<(), ReactError>;
pub type SubscribeResult = Result<(), SubscribeError>;

mod conversion;
pub mod resource;
pub mod build;

pub trait Machine {
    fn act(&mut self) -> ActResult;

    fn react(&mut self, ctx: &ReactContext) -> ReactResult { 
        _ = ctx; 
        Ok(())
    }

    fn subscribe(&mut self, ctx: &SubscribeContext) -> SubscribeResult {
        _ = ctx;
        Err(SubscribeError::OperationNotSupported)
    }

    fn unsubscribe(&mut self, ident: MachineIdentificationUnique) { _ = ident }
}
