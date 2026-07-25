use std::any::Any;

pub use qitech_framework_common::MachineIdentificationUnique;

pub mod error;
use error::ActResult;
use error::ReactResult;
use error::SubscribeError;
use error::SubscribeResult;

mod build;
pub use build::BuildContext;

mod subscribe;
use qitech_framework_common::MachinesReport;
pub use subscribe::ReactContext;
pub use subscribe::SubscribeContext;

pub(crate) mod hardware;
pub use hardware::Hardware;

pub mod resource;
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

#[derive(Default)]
pub(crate) struct Resources {
    pub config_properties: ConfigPropertyManager,
    pub state_properties: StatePropertyManager,
    pub measurements: MeasurementManager,
    pub commands: CommandManager,
    pub events: EventManager,
}

impl Resources {
    pub fn clear_machine(&mut self, ident: MachineIdentificationUnique) {
        self.config_properties.unregister_machine(ident);
        self.state_properties.unregister_machine(ident);
        self.measurements.unregister_machine(ident);
        self.commands.unregister_machine(ident);
        self.events.unregister_machine(ident);
    }

    pub fn init_report(&mut self, report: &mut MachinesReport) {
        self.config_properties.drain_journal(|entry| {
            report.config_mutations.push(entry);
        });

        self.state_properties.drain_journal(|entry| {
            report.state_mutations.push(entry);
        });

        self.measurements.drain_measurements(|entry| {
            report.measurements.push(entry);
        });

        self.commands.drain_journal(|entry| {
            report.commands.push(entry);
        });

        self.events.drain_journal(|entry| {
            report.events.push(entry);
        });
    }
}
