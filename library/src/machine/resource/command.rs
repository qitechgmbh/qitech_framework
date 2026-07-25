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
use crate::machine::resource::ResourceKind;
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<M, A>(
        &mut self,
        ident: MachineIdentificationUnique,
        path: &'static str,
        options: RegisterOptions<M, A>,
    ) -> RegisterResult<Handle>
    where
        M: Machine + 'static,
        A: serde::de::DeserializeOwned + 'static,
    {
        let key = Key {
            ident,
            path,
            postfix: "",
        };

        let Some(execute) = options.execute else {
            return Err(RegisterError {
                resource_kind: ResourceKind::Command,
                resource_path: path,
                error_kind: RegisterErrorKind::MissingRequiredField("execute"),
            });
        };

        if self.registry.contains_key(&key) {
            return Err(RegisterError {
                resource_kind: ResourceKind::Command,
                resource_path: path,
                error_kind: RegisterErrorKind::Duplicate,
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
                resource_kind: ResourceKind::Command,
                resource_path: path,
                machine_ident: ident,
            },
        };

        self.registry.insert(key, Entry { enabled, execute });
        Ok(handle)
    }

    pub fn unregister_machine(&mut self, ident: MachineIdentificationUnique) {
        self.registry.retain(|key, _| key.ident != ident);
    }

    pub fn execute(
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
        let handle = self.journal.new_handle();

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

            handle.append(entry);
            Ok(())
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

    pub fn drain_journal(&mut self, f: impl FnMut(MachineCommandCall)) {
        self.journal.drain_with(f);
    }
}

impl Default for Manager {
    fn default() -> Self {
        Self {
            registry: Default::default(),
            journal: Journal::new(),
        }
    }
}

struct Entry {
    enabled: Rc<RefCell<bool>>,
    execute: ExecuteFn,
}

pub struct RegisterOptions<M, A> {
    pub disabled: bool,
    
    #[allow(clippy::type_complexity)]
    pub execute: Option<fn(&mut M, A) -> Result<(), ExecuteError>>,
}

// you piece of shit compiler too retarded to use the derive properly... FUCK. YOU.
impl<M, A> Default for RegisterOptions<M, A> {
    fn default() -> Self {
        Self { disabled: Default::default(), execute: Default::default() }
    }
}

// --- errors ---
#[derive(Debug)]
pub enum ExecuteError {
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
    use serde::Deserialize;
    use serde::Serialize;

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

        let mut r = Manager::new();

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
        let mut handle = r.register(ident, "simple", RegisterOptions { 
            disabled: false, 
            execute: Some(TestMachine::simple_command),
        })?;
        handle.set_enabled(false)?;
        handle.set_enabled(true)?;

        // --- complex ---
        let mut handle = r.register(ident, "not.simple", RegisterOptions { 
            disabled: false,
            execute: Some(TestMachine::complex_command),
        })?;
        handle.set_enabled(false)?;
        handle.set_enabled(true)?;

        // --- execute simple ---
        r.execute(ident, &mut TestMachine, "simple", "null")?;

        // --- execute complex ---
        let args = ComplexCommandArgs {
            a: 2.0,
            b: 5,
            c: false,
            d: "Hello World".to_string(),
        };
        let args = &serde_json::to_string(&args)?;
        r.execute(ident, &mut TestMachine, "not.simple", args)?;

        Ok(())
    }
}
