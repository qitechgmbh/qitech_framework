use qitech_framework_core::ScalarValue;
use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::report::ConfigPropertyWriteError;
use qitech_framework_core::request::WriteConfigPropertyError;

use crate::machine::Machine;
use crate::machine::error::ActResult;
use crate::machine::error::CommandExecuteResult;

pub(crate) struct MachineInstance {
    pub ident: MachineIdentificationUnique,

    pub machine: Box<dyn Machine>,
    pub config_props: Vec<ConfigPropertyHandle>,
    pub commands: Vec<CommandHandle>,
}

impl MachineInstance {
    pub fn act(&mut self) -> ActResult {
        self.machine.act()
    }

    pub fn set_config_property(
        &mut self,
        path: &str,
        request_id: u64,
        value: ScalarValue,
    ) -> Result<SetConfigPropertySuccess, WriteConfigPropertyError> {
        let Some(instance) = self.config_props.iter().find(|prop| prop.resource == path) else {
            return Err(WriteConfigPropertyError::ResourceNotFound);
        };

        // --- execute write ---
        let changed = (instance.write)(request_id, value)?;

        // --- invoke callback if installed ---
        let callback_result = instance
            .on_changed
            .as_ref()
            .map(|on_changed| on_changed(self.machine.as_mut()));

        Ok(SetConfigPropertySuccess {
            changed,
            callback_result,
        })
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
    resource: &'static str,
    write: Box<dyn Fn(u64, ScalarValue) -> Result<bool, ConfigPropertyWriteError>>,
    on_changed: Option<Box<dyn Fn(&mut dyn Machine) -> ActResult>>,
}

pub type ConfigPropertyWriteFn = Box<dyn Fn(ScalarValue) -> Result<bool, ConfigPropertyWriteError>>;

pub struct SetConfigPropertySuccess {
    changed: bool,
    callback_result: Option<ActResult>,
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
