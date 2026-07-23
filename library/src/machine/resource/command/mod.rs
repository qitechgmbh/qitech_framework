use std::{borrow::Cow, cell::RefCell, collections::HashMap, rc::{Rc, Weak}};
use chrono::Utc;
use qitech_framework_common::{MachineCommandCall, MachineIdentificationUnique, OperationResult};
use crate::machine::{Machine, resource::{Journal, Key, error::{RegisterError, RegisterErrorKind, RegisterResult}, kind::Kind}};

type ExecuteFn = Box<dyn Fn(&mut dyn Machine, &str) -> Result<(), CommandExecuteError>>;

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

    pub(crate) fn execute(
        &mut self,
        target: MachineIdentificationUnique, 
        path: &str, 
        machine: &mut dyn Machine,
        args: &str,
    ) -> Result<(), CommandExecuteError> {
        let key = Key { ident: target, path, postfix: "" };

        let mut entry = MachineCommandCall {
            target,
            resource_path: Cow::Owned(path.to_owned()),
            arguments: args.to_string(),
            timestamp: Utc::now(),
            result: OperationResult::Success,
        };

        let Some(Entry { enabled, execute }) = self.registry.get(&key) else {
            entry.result = OperationResult::Failure;
            
            if self.journal.borrow_mut().push(entry).is_err() {
                return Err(CommandExecuteError::JournalFull);
            }

            return Err(CommandExecuteError::NotFound);
        };

        if !*enabled.borrow() {
            entry.result = OperationResult::Failure;

            if self.journal.borrow_mut().push(entry).is_err() {
                return Err(CommandExecuteError::JournalFull);
            }

            return Err(CommandExecuteError::Disabled);
        }

        if let Err(e) = (execute)(machine, args) {
            entry.result = OperationResult::Failure;

            if self.journal.borrow_mut().push(entry).is_err() {
                return Err(CommandExecuteError::JournalFull);
            }

            return Err(e);
        }

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

pub enum CommandExecuteError {
    UnexpectedMachineType {
        expected: &'static str,
        received: &'static str,
    },
    JournalFull,
    Disabled,
    NotFound,
    ParsingError(serde_json::Error),
    ExecutionError(String)
}

/// Handle Expired
pub struct CommandHandleError;