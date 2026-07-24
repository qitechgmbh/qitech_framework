use std::any::Any;

use qitech_framework_common::MachineIdentificationUnique;

use crate::machine::error::ActResult;
use crate::machine::error::ReactResult;
use crate::machine::error::SubscribeError;
use crate::machine::error::SubscribeResult;
use crate::machine::resource::config_property;
use crate::machine::resource::measurement;
use crate::machine::resource::state_property;

pub mod bounds;

pub mod build;
pub use build::MachineBuild;

pub mod context;
pub mod error;
pub mod resource;

// mod types;
// pub use types::Hardware;
// pub use types::ActError;
// pub use types::ActErrorKind;
// pub use types::ReactContext;
// pub use types::ReactError;
// pub use types::ReactErrorKind;
// pub use types::SubscribeContext;
// pub use types::SubscribeError;

// pub type ActResult = Result<(), ActError>;
// pub type ReactResult = Result<(), ReactError>;
// pub type SubscribeResult = Result<(), SubscribeError>;

// mod conversion;
// pub mod resource;
//
// mod build;
// pub use build::BuildContext;
// pub use build::BuildResult;
// pub use build::BuildError;
// pub use build::Build;

pub trait Machine: Any {
    fn act(&mut self) -> ActResult;

    fn react(&mut self, ctx: &ReactContext) -> ReactResult {
        _ = ctx;
        Ok(())
    }

    fn subscribe(&mut self, ctx: &SubscribeContext) -> SubscribeResult {
        _ = ctx;
        Err(SubscribeError::OperationNotSupported)
    }

    fn unsubscribe(&mut self, ident: MachineIdentificationUnique) {
        _ = ident
    }
}

pub struct ReactContext<'a> {
    pub config: config_property::Reader<'a>,
    pub state: state_property::Reader<'a>,
    pub measurements: measurement::Reader<'a>,
}

pub struct SubscribeContext<'a> {
    pub ident: MachineIdentificationUnique,
    pub config: config_property::Resolver<'a>,
    pub state: state_property::Resolver<'a>,
    pub measurements: measurement::Resolver<'a>,
}
