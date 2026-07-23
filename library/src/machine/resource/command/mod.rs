use std::{cell::RefCell, collections::HashMap, rc::{Rc, Weak}};
use qitech_framework_common::{MachineCommandCall, MachineIdentificationUnique};
use crate::machine::{Machine, resource::{Journal, Key, error::{RegisterError, RegisterErrorKind, RegisterResult}, kind::Kind}};

type ExecuteFn = Box<dyn Fn(&mut dyn Machine, &str) -> Result<(), CommandError>>;

pub struct Manager {
    registry: HashMap<Key<'static>, Entry>,
    journal: Rc<RefCell<Journal<MachineCommandCall>>>,
}

impl Manager {
    pub(crate) fn register(
        &mut self,
        ident: MachineIdentificationUnique, 
        path: &'static str,
        execute: ExecuteFn,
    ) -> RegisterResult<CommandHandle> {
        let key = Key { ident, path, postfix: "" };

        if self.registry.contains_key(&key) {
            return Err(RegisterError {
                resource_kind: Kind::Command,
                resource_path: path,
                kind: RegisterErrorKind::AlreadyRegistered,
            });
        }

        self.registry.insert(key, Entry { enabled: Default::default(), execute });
        todo!()
    }

    pub(crate) fn invoke(
        &mut self,
        target: MachineIdentificationUnique, 
        path: &str, 
        machine: &mut dyn Machine,
        args: &str,
    ) -> Result<(), String> {
        let key = Key { ident: target, path, postfix: "" };
        let Some(entry) = self.registry.get(&key) else {
            return Err("No such entry".to_string());
        };

        (entry.execute)(machine, args);
        Ok(())
    }
}

pub struct Entry {
    enabled: Rc<RefCell<bool>>,
    execute: ExecuteFn,
}

pub struct CommandHandle {
    enabled: Weak<RefCell<bool>>,
}

impl CommandHandle {
    pub fn set_enabled(&mut self, value: bool) -> Result<(), CommandHandleError> {
        let Some(handle) = self.enabled.upgrade() else {
            return Err(CommandHandleError);
        };

        *handle.borrow_mut() = value;
        Ok(())
    }
}

pub enum CommandError {
    UnexpectedMachineType {
        expected: &'static str,
        received: &'static str,
    },
    ParsingError(serde_json::Error),
    ExecutionError(String)
}

/// Handle Expired
pub struct CommandHandleError;