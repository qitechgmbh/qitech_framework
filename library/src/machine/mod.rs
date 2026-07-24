use std::any::Any;

pub use qitech_framework_common::MachineIdentificationUnique;
use qitech_framework_common::MachineSchema;

use crate::machine::error::ActResult;
use crate::machine::error::GrantError;
use crate::machine::error::GrantResult;
use crate::machine::error::ReactResult;
pub use crate::machine::resource::ConfigPropertyReader;
pub use crate::machine::resource::ConfigPropertyReaderHandle;
pub use crate::machine::resource::ConfigPropertyResolver;
pub use crate::machine::resource::MeasurementReader;
pub use crate::machine::resource::MeasurementRegistrar;
pub use crate::machine::resource::MeasurementResolver;
pub use crate::machine::resource::StatePropertyReader;
pub use crate::machine::resource::StatePropertyReaderHandle;
pub use crate::machine::resource::StatePropertyResolver;

pub mod bounds;

pub mod error;
pub mod resource;

mod registry;

pub trait Machine: Any {
    fn act(&mut self) -> ActResult;

    fn react(&mut self, ctx: &ReactContext) -> ReactResult {
        _ = ctx;
        Ok(())
    }

    fn on_grant_offered(&mut self, ctx: &GrantContext) -> GrantResult {
        _ = ctx;
        Err(GrantError::Rejected)
    }

    fn on_grant_revoked(&mut self, ident: MachineIdentificationUnique) {
        _ = ident
    }
}

pub trait MachineBuild: Sized {
    fn build(ctx: BuildContext<'_>) -> error::BuildResult<Self>;
}

pub trait MachineInterface {
    const SCHEMA: &'static str;
}

pub struct BuildContext<'a> {
    // pub hardware: HardwareSet,
    // pub config: ConfigPropertyReader<'a>,
    // pub state: StatePropertyReader<'a>,
    pub measurements: MeasurementRegistrar<'a>,
}

pub struct ReactContext<'a> {
    pub config: ConfigPropertyReader<'a>,
    pub state: StatePropertyReader<'a>,
    pub measurements: MeasurementReader<'a>,
}

pub struct GrantContext<'a> {
    pub source: MachineIdentificationUnique,
    pub config: ConfigPropertyResolver<'a>,
    pub state: StatePropertyResolver<'a>,
    pub measurements: MeasurementResolver<'a>,
}
