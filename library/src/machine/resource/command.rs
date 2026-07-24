use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::rc::Weak;

use chrono::Utc;
use qitech_framework_common::MachineCommandCall;
use qitech_framework_common::MachineIdentificationUnique;
use qitech_framework_common::OperationResult;

use crate::machine::Machine;
use crate::machine::resource::Journal;
use crate::machine::resource::Key;
use crate::machine::resource::Kind;
use crate::machine::resource::error::HandleError;
use crate::machine::resource::error::RegisterError;
use crate::machine::resource::error::RegisterErrorKind;
use crate::machine::resource::error::RegisterResult;

type ExecuteFn = Box<dyn Fn(&mut dyn Machine, &str) -> Result<(), ExecuteError>>;

pub struct Handle {
    resource_path: &'static str,
    machine_ident: MachineIdentificationUnique,
    enabled: Weak<RefCell<bool>>,
}

impl Handle {
    pub fn set_enabled(&mut self, value: bool) -> Result<(), HandleError> {
        let Some(handle) = self.enabled.upgrade() else {
            return Err(HandleError {
                resource_kind: Kind::Command,
                resource_path: self.resource_path,
                machine_ident: self.machine_ident,
            });
        };

        *handle.borrow_mut() = value;
        Ok(())
    }
}

pub struct Manager {
    registry: HashMap<Key<'static>, Entry>,
    journal: Journal<MachineCommandCall>,
}

impl Manager {
    pub(crate) fn register(
        &mut self,
        ident: MachineIdentificationUnique,
        path: &'static str,
        execute: ExecuteFn,
    ) -> RegisterResult<Handle> {
        let key = Key {
            ident,
            path,
            postfix: "",
        };

        if self.registry.contains_key(&key) {
            return Err(RegisterError {
                resource_kind: Kind::Command,
                resource_path: path,
                kind: RegisterErrorKind::Duplicate,
            });
        }

        self.registry.insert(
            key,
            Entry {
                enabled: Default::default(),
                execute,
            },
        );
        todo!()
    }

    pub(crate) fn execute(
        &mut self,
        target: MachineIdentificationUnique,
        machine: &mut dyn Machine,
        path: &str,
        args: &str,
    ) -> Result<(), ExecuteError> {
        let key = Key {
            ident: target,
            path,
            postfix: "",
        };
        let handle = self.journal.init_handle();

        let finish = |err: Result<(), ExecuteError>| -> Result<(), ExecuteError> {
            let result = if err.is_ok() {
                OperationResult::Success
            } else {
                OperationResult::Failure
            };

            let entry = MachineCommandCall {
                target,
                resource_path: Cow::Owned(path.to_owned()),
                arguments: args.to_string(),
                timestamp: Utc::now(),
                result,
            };

            match handle.append(entry) {
                Ok(()) => err,
                Err(_) => Err(ExecuteError::JournalFull),
            }
        };

        let Some(Entry { enabled, execute }) = self.registry.get(&key) else {
            return finish(Err(ExecuteError::NotFound));
        };

        if !*enabled.borrow() {
            return finish(Err(ExecuteError::Disabled));
        }

        if let Err(e) = (execute)(machine, args) {
            return finish(Err(e));
        }

        finish(Ok(()))
    }
}

struct Entry {
    enabled: Rc<RefCell<bool>>,
    execute: ExecuteFn,
}

#[derive(Debug)]
pub(crate) enum ExecuteError {
    JournalFull,
    UnexpectedMachineType {
        expected: &'static str,
        received: &'static str,
    },
    Disabled,
    NotFound,
    ParsingError(serde_json::Error),
    ExecutionError(String),
}

impl fmt::Display for ExecuteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JournalFull => write!(f, "command journal is full"),
            Self::UnexpectedMachineType { expected, received } => {
                write!(
                    f,
                    "unexpected machine type: expected `{expected}`, received `{received}`"
                )
            }
            Self::Disabled => write!(f, "command is disabled"),
            Self::NotFound => write!(f, "command not found"),
            Self::ParsingError(err) => write!(f, "failed to parse command arguments: {err}"),
            Self::ExecutionError(msg) => write!(f, "command execution failed: {msg}"),
        }
    }
}

impl std::error::Error for ExecuteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ParsingError(err) => Some(err),
            _ => None,
        }
    }
}
