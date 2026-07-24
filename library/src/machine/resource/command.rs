use std::any;
use std::any::Any;
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
    error: HandleError,
    enabled: Weak<RefCell<bool>>,
}

impl Handle {
    pub fn set_enabled(&mut self, value: bool) -> Result<(), HandleError> {
        let Some(handle) = self.enabled.upgrade() else {
            return Err(self.error);
        };

        *handle.borrow_mut() = value;
        Ok(())
    }
}

// --- resource managment ---
pub struct Manager {
    registry: HashMap<Key<'static>, Entry>,
    journal: Journal<MachineCommandCall>,
}

impl Manager {
    pub(crate) fn new() -> Self {
        Self {
            registry: Default::default(),
            journal: Journal::new(),
        }
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

/// --- registering ---
pub struct Registrar<'a> {
    manager: &'a mut Manager,
    machine: MachineIdentificationUnique,
}

impl<'a> Registrar<'a> {
    pub(crate) fn new(manager: &'a mut Manager, machine: MachineIdentificationUnique) -> Self {
        Self { manager, machine }
    }

    pub(crate) fn register<M, A>(
        &mut self,
        path: &'static str,
        execute: fn(&mut M, A) -> Result<(), ExecuteError>,
    ) -> RegisterResult<Handle>
    where
        M: Machine + 'static,
        A: serde::de::DeserializeOwned + 'static,
    {
        let reg = &mut self.manager.registry;

        let key = Key {
            ident: self.machine,
            path,
            postfix: "",
        };

        if reg.contains_key(&key) {
            return Err(RegisterError {
                resource_kind: Kind::Command,
                resource_path: path,
                kind: RegisterErrorKind::Duplicate,
            });
        }

        let execute = Box::new(move |machine: &mut dyn Machine, bytes: &str| {
            let machine_type_name = any::type_name_of_val(machine);
            let any: &mut dyn Any = machine;

            let machine = any
                .downcast_mut::<M>()
                .ok_or(ExecuteError::UnexpectedMachineType {
                    expected: any::type_name::<M>(),
                    received: machine_type_name,
                })?;

            let args: A = match serde_json::from_str(bytes) {
                Ok(v) => v,
                Err(e) => return Err(ExecuteError::ParsingError(e)),
            };

            execute(machine, args)
        });

        let enabled = Rc::default();

        let handle = Handle {
            enabled: Rc::downgrade(&enabled),
            error: HandleError {
                resource_kind: Kind::Command,
                resource_path: path,
                machine_ident: self.machine,
            },
        };

        reg.insert(key, Entry { enabled, execute });

        Ok(handle)
    }
}

// --- errors ---
#[derive(Debug)]
pub enum ExecuteError {
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

// --- testing ---
#[cfg(test)]
mod test {
    use qitech_framework_common::MachineIdentification;
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::machine::error::ActResult;

    #[test]
    pub fn register_and_use() -> anyhow::Result<()> {
        let ident = MachineIdentificationUnique {
            identification: MachineIdentification {
                vendor_id: 0,
                machine_id: 0,
            },
            serial: 0,
        };

        let mut mgr = Manager::new();
        let mut r = Registrar::new(&mut mgr, ident);

        struct TestMachine;

        impl Machine for TestMachine {
            fn act(&mut self) -> ActResult {
                Ok(())
            }
        }

        impl TestMachine {
            fn simple_command(&mut self, _args: ()) -> Result<(), ExecuteError> {
                println!("Hello World!");
                Ok(())
            }

            fn complex_command(&mut self, args: ComplexCommandArgs) -> Result<(), ExecuteError> {
                assert_eq!(args.a, 2.0);
                assert_eq!(args.b, 5);
                assert!(!args.c);
                assert_eq!(&args.d, "Hello World");
                Ok(())
            }
        }

        #[derive(Serialize, Deserialize)]
        struct ComplexCommandArgs {
            a: f64,
            b: i64,
            c: bool,
            d: String,
        }

        // --- simple ---
        let mut handle = r.register("simple", TestMachine::simple_command)?;
        handle.set_enabled(false)?;
        handle.set_enabled(true)?;

        // --- complex ---
        let mut handle = r.register("not.simple", TestMachine::complex_command)?;
        handle.set_enabled(false)?;
        handle.set_enabled(true)?;

        // --- execute simple ---
        mgr.execute(ident, &mut TestMachine, "simple", "null")?;

        // --- execute complex ---
        let args = ComplexCommandArgs {
            a: 2.0,
            b: 5,
            c: false,
            d: "Hello World".to_string(),
        };
        let args = &serde_json::to_string(&args)?;
        mgr.execute(ident, &mut TestMachine, "not.simple", args)?;

        Ok(())
    }
}
