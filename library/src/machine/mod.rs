use std::any::Any;
use std::any::TypeId;
use std::ptr::NonNull;

pub use qitech_framework_core::ident::MachineIdentification;
pub use qitech_framework_core::ident::MachineIdentificationUnique;
pub use qitech_framework_core::request::SubscribeError;

use crate::machine::error::CommandExecuteResult;
pub use crate::resource::ConfigProperty;
use crate::resource::LifetimeTokenProvider;
pub use crate::resource::Measurement;
pub use crate::resource::StateProperty;

pub mod error;
use error::ActResult;
use error::BuildResult;
use error::SubscribeResult;

mod build;
pub use build::BuildContext;

mod subscribe;
pub use subscribe::RemoteProperty;
pub use subscribe::SubscribeContext;

pub(crate) mod hardware;
pub use hardware::Hardware;

mod command;

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
    pub lifetime: LifetimeTokenProvider,

    pub machine: Box<dyn Machine>,
    pub configs: 
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
pub struct ConfigPropertyHandle {
    handle: PropertyHandle,
    resource: &'static str,

    write: Box<dyn Fn(&PropertyHandle) -> ActResult>,
    on_changed: Option<Box<dyn Fn(&mut dyn Machine) -> ActResult>>,
}

pub(crate) struct PropertyHandle {
    type_id: TypeId,
    type_name: &'static str,
    p_value: NonNull<()>,
    p_cache: NonNull<()>,
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
