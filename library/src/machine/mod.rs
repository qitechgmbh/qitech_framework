use std::any::Any;

pub use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::report::MachinesReport;

pub mod error;
use error::ActResult;
use error::SubscribeError;
use error::SubscribeResult;

mod build;
pub use build::BuildContext;

mod subscribe;
pub use subscribe::SubscribeContext;

pub(crate) mod hardware;
pub use hardware::Hardware;

pub mod resource;
pub use resource::CommandHandle;
pub use resource::IntoExecuteFn;
pub use resource::SubscribedEvent;
pub use resource::conversion::BoundedMeta;
pub use resource::conversion::TypeWrapper;
pub use resource::subscription::SubscribedProperty;

use crate::machine::resource::CommandManager;
use crate::machine::resource::ConfigPropertyManager;
use crate::machine::resource::EventManager;
use crate::machine::resource::MeasurementManager;
use crate::machine::resource::StatePropertyManager;

pub trait Machine: Any {
    /// defines the update cycle of a machine
    fn act(&mut self) -> ActResult;

    /// allows a machine to sync remote resources (from subscriptions)
    fn subscribe(&mut self, ctx: SubscribeContext) -> SubscribeResult<()> {
        _ = ctx;
        Err(SubscribeError::Rejected)
    }

    /// called when the machine is notified a subscription is canceled
    fn unsubscribe(&mut self, ident: MachineIdentificationUnique) {
        _ = ident
    }
}

pub trait MachineBuild: Sized {
    fn build(ctx: BuildContext<'_>) -> error::BuildResult<Self>;
}

pub trait MachineInterface {
    // TODO: parse schema at compile time and expose that instead ...
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

    pub fn sync_caches(&mut self) {
        self.config_properties.sync_cache();
        self.state_properties.sync_cache();
        self.measurements.sync_cache();
        self.events.sync_cache();
    }

    pub fn remove_subscription(
        &mut self,
        provider: MachineIdentificationUnique,
        subscriber: MachineIdentificationUnique,
    ) {
        self.config_properties
            .remove_subscription(provider, subscriber);
        self.state_properties
            .remove_subscription(provider, subscriber);
        self.measurements.remove_subscription(provider, subscriber);
        self.events.remove_subscription(provider, subscriber);
    }

    pub fn extract_report(&mut self, report: &mut MachinesReport) {
        self.config_properties.drain_journal(|entry| {
            report.config_mutations.push(entry);
        });

        self.state_properties.drain_journal(|entry| {
            report.state_mutations.push(entry);
        });

        self.measurements.iter(|entry| {
            report.measurements.push(entry);
        });

        /*
        self.commands.drain_journal(|entry| {
            report.commands.push(entry);
        });

        self.events.drain_journal(|entry| {
            report.events.push(entry);
        });
        */
    }
}
