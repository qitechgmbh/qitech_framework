use std::any::Any;
use std::any::TypeId;
use std::any::type_name;
use std::ptr::NonNull;
use std::rc::Rc;
use std::rc::Weak;

pub use qitech_framework_core::ident::MachineIdentification;
pub use qitech_framework_core::ident::MachineIdentificationUnique;
pub use qitech_framework_core::request::SubscribeError;

use crate::machine::error::CommandExecuteResult;
use crate::machine::instance::ConfigPropertyWriteFn;

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
mod state_property;
pub use state_property::StateProperty;
mod property_registry;
pub use property_registry::PropertyRegistry;
pub use property_registry::PropertyRegistrar;

pub(crate) mod hardware;
pub use hardware::Hardware;

mod command;
mod constraints;
mod conversion;
mod instance;
mod measurement;

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
    fn build(ctx: &mut BuildContext) -> BuildResult<Self>;
}

pub trait MachineDescriptor {
    const SCHEMA: &'static str;
    const IDENTIFICATION: MachineIdentification;
}

// --- instance ---
pub(crate) struct MachineInstance {
    pub ident: MachineIdentificationUnique,
    pub machine: Box<dyn Machine>,
    pub configs: Vec<ConfigPropertyHandle>,
    pub commands: Vec<CommandHandle>,
}

impl MachineInstance {
    pub fn act(&mut self) -> ActResult {
        self.machine.act()
    }

    pub fn update_can_execute(&mut self) {
        for command in &mut self.commands {
            let Some(func) = &command.can_execute_fn else {
                continue;
            };

            let can_execute = (func)(self.machine.as_ref());

            if can_execute != command.can_execute_prev {
                // changed -> record
            }

            command.can_execute_prev = can_execute;
        }
    }

    pub fn execute_command(&mut self, resource: &'static str) -> CommandExecuteResult {
        let Some(command) = self.commands.iter().find(|cmd| cmd.resource == resource) else {
            panic!("TODO: return error");
        };

        if let Some(func) = &command.can_execute_fn {
            let can_execute = (func)(self.machine.as_ref());

            if can_execute != command.can_execute_prev {
                // changed -> record
            }

            if !can_execute {
                panic!("TODO: return error -> Disabled");
            }
        }

        (command.execute_fn)(self.machine.as_mut())
    }
}

// --- config handle ---
pub(crate) struct ConfigPropertyHandle {
    resource: &'static str,
    write: ConfigPropertyWriteFn,
    on_changed: Option<Box<dyn Fn(&mut dyn Machine) -> ActResult>>,
}

impl ConfigPropertyHandle {
    pub(crate) fn new(
        resource: &'static str,
        write: ConfigPropertyWriteFn,
        on_changed: Option<Box<dyn Fn(&mut dyn Machine) -> ActResult>>,
    ) -> Self {
        Self { resource, write, on_changed }
    }
}

pub(crate) struct PropertyHandle<T = ()> {
    type_id: TypeId,
    type_name: &'static str,
    p_value: NonNull<T>,
    p_cache: NonNull<T>,
}

impl PropertyHandle<()> {
    pub(crate) fn downcast<T: 'static>(&self) -> PropertyHandle<T> {
        assert_eq!(
            self.type_id,
            TypeId::of::<T>(),
            "property type mismatch: expected {}, got {}",
            self.type_name,
            type_name::<T>(),
        );

        PropertyHandle {
            type_id: self.type_id,
            type_name: self.type_name,
            p_value: self.p_value.cast(),
            p_cache: self.p_cache.cast(),
        }
    }
}

// --- command handle ---
pub struct CommandHandle {
    resource: &'static str,
    can_execute_prev: bool,
    can_execute_fn: Option<Box<dyn Fn(&dyn Machine) -> bool>>,
    execute_fn: Box<dyn Fn(&mut dyn Machine) -> Result<(), String>>,
}

impl CommandHandle {
    pub fn can_execute(&self, machine: &dyn Machine) -> bool {
        self.can_execute_fn
            .as_ref()
            .map(|f| f(machine))
            .unwrap_or(true)
    }

    pub fn execute(&self, machine: &mut dyn Machine) -> Result<(), String> {
        (self.execute_fn)(machine)
    }
}

// --- key ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceKey {
    pub ident: MachineIdentificationUnique,
    pub path: &'static str,
}

// --- misc ---
#[derive(Debug, Default)]
pub struct LifetimeTokenProvider {
    inner: Rc<()>,
}

impl LifetimeTokenProvider {
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
