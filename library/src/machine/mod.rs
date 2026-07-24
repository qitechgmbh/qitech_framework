use std::any::Any;

pub use qitech_framework_common::MachineIdentificationUnique;

use crate::machine::error::ActResult;
use crate::machine::error::GrantError;
use crate::machine::error::GrantResult;
use crate::machine::error::ReactResult;
pub use crate::machine::resource::CommandHandle;
use crate::machine::resource::CommandRegistrar;
pub use crate::machine::resource::ConfigPropertyReader;
pub use crate::machine::resource::ConfigPropertyReaderHandle;
use crate::machine::resource::ConfigPropertyRegistrar;
pub use crate::machine::resource::ConfigPropertyResolver;
pub use crate::machine::resource::EventEmitter;
use crate::machine::resource::EventRegistrar;
pub use crate::machine::resource::Measurement;
pub use crate::machine::resource::MeasurementReader;
pub use crate::machine::resource::MeasurementRegisterOptions;
pub use crate::machine::resource::MeasurementRegistrar;
pub use crate::machine::resource::MeasurementResolver;
pub use crate::machine::resource::StateProperty;
pub use crate::machine::resource::StatePropertyReader;
pub use crate::machine::resource::StatePropertyReaderHandle;
pub use crate::machine::resource::StatePropertyRegistrar;
pub use crate::machine::resource::StatePropertyResolver;

pub mod bounds;
pub mod error;
pub(crate) mod resource;

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
    pub config: ConfigPropertyRegistrar<'a>,
    pub state: StatePropertyRegistrar<'a>,
    pub measurements: MeasurementRegistrar<'a>,
    pub commands: CommandRegistrar<'a>,
    pub events: EventRegistrar<'a>,
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
