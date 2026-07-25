use std::any::Any;

pub use qitech_framework_common::MachineIdentificationUnique;

pub mod error;
use error::ActResult;
use error::ReactResult;
use error::SubscribeError;
use error::SubscribeResult;

mod hardware;
use hardware::HardwareAccessor;

mod build;

pub(crate) mod resource;
use qitech_lib::ethercat_hal::EtherCATThreadChannel;
use resource::CommandHandle;
use resource::CommandRegistrar;
use resource::ConfigPropertyReader;
use resource::ConfigPropertyReaderHandle;
use resource::ConfigPropertyRegistrar;
use resource::ConfigPropertyResolver;
use resource::EventEmitter;
use resource::EventRegistrar;
use resource::Measurement;
use resource::MeasurementReader;
use resource::MeasurementRegisterOptions;
use resource::MeasurementRegistrar;
use resource::MeasurementResolver;
use resource::StateProperty;
use resource::StatePropertyReader;
use resource::StatePropertyReaderHandle;
use resource::StatePropertyRegistrar;
use resource::StatePropertyResolver;

use crate::machine::hardware::Hardware;
use crate::machine::resource::CommandManager;
use crate::machine::resource::ConfigPropertyManager;
use crate::machine::resource::EventManager;
use crate::machine::resource::MeasurementManager;
use crate::machine::resource::StatePropertyManager;

pub trait Machine: Any {
    fn act(&mut self) -> ActResult;

    fn react(&mut self, ctx: &ReactContext) -> ReactResult {
        _ = ctx;
        Ok(())
    }

    fn subscribe(&mut self, ctx: &SubscribeContext) -> SubscribeResult {
        _ = ctx;
        Err(SubscribeError::Rejected)
    }

    fn unsubscribe(&mut self, ident: MachineIdentificationUnique) {
        _ = ident
    }
}

pub trait MachineBuild: Sized {
    fn build(ctx: BuildContext<'_>) -> error::BuildResult<Self>;
}

pub trait MachineInterface {
    const SCHEMA: &'static str;
}

pub struct ReactContext<'a> {
    resources: &'a mut Resources,
}

impl<'a> ReactContext<'a> {
    pub fn read(&self, handle: ()) {}
}

pub struct SubscribeContext<'a> {
    source: MachineIdentificationUnique,
    resources: &'a mut Resources,
}

impl<'a> SubscribeContext<'a> {
    pub fn source(&self) -> MachineIdentificationUnique {
        self.source
    }
}

pub(crate) struct Resources {
    config_properties: ConfigPropertyManager,
    state_properties: StatePropertyManager,
    measurements: MeasurementManager,
    commands: CommandManager,
    events: EventManager,
}

impl Resources {
    pub fn clear_machine(&mut self, ident: MachineIdentificationUnique) {
        self.config_properties.unregister_machine(ident);
        self.state_properties.unregister_machine(ident);
        self.measurements.unregister_machine(ident);
        self.commands.unregister_machine(ident);
        self.events.unregister_machine(ident);
    }
}
