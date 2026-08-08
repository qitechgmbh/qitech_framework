use std::any::Any;
use std::rc::Rc;
use std::rc::Weak;

pub use qitech_framework_core::ident::MachineIdentification;
pub use qitech_framework_core::ident::MachineIdentificationUnique;
pub use qitech_framework_core::request::MachineSubscribeError;
pub use qitech_framework_core::report::OperationCapability;

mod bump_allocator;
use bump_allocator::BumpAllocator;
use bump_allocator::BumpAllocatorMark;

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

mod state_property;
pub use state_property::StateProperty;

mod property_registry;
pub use property_registry::PropertyRegistrar;
pub use property_registry::PropertyRegistry;

pub(crate) mod hardware;
pub use hardware::Hardware;

mod command;
pub(crate) mod constraints;
pub(crate) mod conversion;
mod measurement;
pub use measurement::Measurement;

mod instance;
pub(crate) use instance::ConfigPropertyChangedCallbackFn;
pub(crate) use instance::ConfigPropertyHandle;
pub(crate) use instance::ConfigPropertyWriteFn;
pub(crate) use instance::MachineInstance;

pub trait Machine: Any {
    /// defines the update cycle of a machine
    fn act(&mut self) -> ActResult;

    // /// allows a machine to sync remote resources (from subscriptions)
    fn subscribe(&mut self, ctx: &mut SubscribeContext) -> SubscribeResult {
        _ = ctx;
        Err(MachineSubscribeError::UnsupportedMachine)
    }

    /// called when the machine is notified a subscription is canceled
    fn unsubscribe(&mut self, ident: MachineIdentificationUnique) {
        _ = ident
    }
}

pub trait MachineBuild: Sized {
    fn build(ctx: &mut BuildContext) -> BuildResult<Self>;
}

pub trait MachineDescriptor {
    const SCHEMA: &'static str;
    const IDENTIFICATION: MachineIdentification;
}

// --- key ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceKey {
    pub ident: MachineIdentificationUnique,
    pub path: &'static str,
}

// --- misc ---
#[derive(Debug, Default)]
pub struct LifetimeTokenOwner {
    inner: Rc<()>,
}

impl LifetimeTokenOwner {
    pub fn new() -> Self {
        Self { inner: Rc::new(()) }
    }

    pub fn new_token(&self) -> LifetimeToken {
        LifetimeToken {
            inner: Rc::downgrade(&self.inner),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LifetimeToken {
    inner: Weak<()>,
}

impl LifetimeToken {
    pub fn expired(&self) -> bool {
        self.inner.upgrade().is_none()
    }
}

// --- resources ---
pub struct ResourceRegistry {
    pub config_properties: PropertyRegistry,
    pub state_properties: PropertyRegistry,
    pub measurements: PropertyRegistry,
}
