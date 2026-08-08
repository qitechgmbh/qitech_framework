use std::collections::HashMap;

use qitech_framework_core::ScalarValue;
use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::report::ConfigPropertyWriteError;

use crate::machine::LifetimeTokenOwner;
use crate::machine::Machine;
use crate::machine::SubscribeContext;
use crate::machine::error::ActResult;
use crate::machine::error::CommandExecuteResult;
use crate::machine::error::SubscribeResult;

pub(crate) struct MachineInstance {
    pub(crate) ident: MachineIdentificationUnique,
    pub(crate) machine: Box<dyn Machine>,
    pub(crate) configs: HashMap<&'static str, ConfigPropertyHandle>,
    pub(crate) commands: Vec<CommandHandle>,
    pub(crate) subscriptions: HashMap<MachineIdentificationUnique, LifetimeTokenOwner>,
}

impl MachineInstance {
    pub fn act(&mut self) -> ActResult {
        self.machine.act()
    }

    pub fn subscribe(&mut self, ctx: &mut SubscribeContext) -> SubscribeResult {
        self.machine.subscribe(ctx)
    }

    pub fn get_config_handle(&mut self, path: &str) -> Option<&mut ConfigPropertyHandle> {
        self.configs.get_mut(path)
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
pub type ConfigPropertyWriteFn = Box<dyn Fn(ScalarValue) -> Result<bool, ConfigPropertyWriteError>>;
pub type ConfigPropertyChangedCallbackFn = Box<dyn Fn(&mut dyn Machine) -> ActResult>;

pub(crate) struct ConfigPropertyHandle {
    pub(crate) write: ConfigPropertyWriteFn,
    pub(crate) on_changed: Option<ConfigPropertyChangedCallbackFn>,
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
