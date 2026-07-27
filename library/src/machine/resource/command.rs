use std::any;
use std::any::Any;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::rc::Weak;

use chrono::Utc;
use qitech_framework_common::MachineCommandCall;
use qitech_framework_common::MachineIdentificationUnique;
use qitech_framework_common::OperationResult;
use serde::de::DeserializeOwned;
use thiserror::Error;

use crate::machine::Machine;
use crate::machine::resource::Journal;
use crate::machine::resource::Key;
use crate::machine::resource::error::RegisterError;
use crate::machine::resource::error::RegisterResult;

pub type ExecuteFn = Box<dyn Fn(&mut dyn Machine, &str) -> Result<(), ExecuteError>>;

pub struct Handle {
    enabled: Weak<RefCell<bool>>,
}

impl Handle {
    pub fn set_enabled(&mut self, value: bool) {
        let handle = self
            .enabled
            .upgrade()
            .expect("Handle outlived manager's entry");
        *handle.borrow_mut() = value;
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

    pub fn register(
        &mut self,
        ident: MachineIdentificationUnique,
        path: &'static str,
        disabled: bool,
        execute: ExecuteFn,
    ) -> RegisterResult<Handle> {
        let key = Key::from_str(ident, path);

        if self.registry.contains_key(&key) {
            return Err(RegisterError::Duplicate);
        }

        let enabled = Rc::<RefCell<bool>>::default();
        *enabled.borrow_mut() = disabled;

        let handle = Handle {
            enabled: Rc::downgrade(&enabled),
        };

        self.registry.insert(key, Entry { enabled, execute });
        Ok(handle)
    }

    pub fn unregister_machine(&mut self, ident: MachineIdentificationUnique) {
        self.registry.retain(|k, _| k.ident != ident);
    }

    pub fn invoke(
        &mut self,
        target: MachineIdentificationUnique,
        machine: &mut dyn Machine,
        path: &str,
        args: &str,
    ) -> Result<(), ExecuteError> {
        let key = Key {
            ident: target,
            path: Cow::Owned(path.to_string()),
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

// pub struct RegisterOptions<M, A> {
//     pub disabled: bool,
//
//     #[allow(clippy::type_complexity)]
//     pub execute: Option<IntoExecuteFn<M, A>>,
// }

// you piece of shit compiler too retarded to use the derive properly... FUCK. YOU.
// impl<M, A> Default for RegisterOptions<M, A> {
//     fn default() -> Self {
//         Self {
//             disabled: Default::default(),
//             execute: Default::default(),
//         }
//     }
// }

// --- trait wank ---
pub trait IntoExecuteFn {
    fn into_execute_fn(self) -> ExecuteFn;
}

impl<M> IntoExecuteFn for fn(&mut M) -> Result<(), String>
where
    M: Machine + 'static,
{
    fn into_execute_fn(self) -> ExecuteFn {
        Box::new(move |machine: &mut dyn Machine, bytes: &str| {
            let machine_type_name = any::type_name_of_val(machine);
            let any: &mut dyn Any = machine;

            let machine = any
                .downcast_mut::<M>()
                .ok_or(ExecuteError::UnexpectedMachineType {
                    expected: any::type_name::<M>(),
                    received: machine_type_name,
                })?;

            // Validate that the caller sent an empty argument payload
            let _: () = serde_json::from_str(bytes).map_err(ExecuteError::ParsingError)?;

            self(machine).map_err(ExecuteError::ExecutionError)
        })
    }
}

impl<M, A> IntoExecuteFn for fn(&mut M, A) -> Result<(), String>
where
    M: Machine + 'static,
    A: DeserializeOwned + 'static,
{
    fn into_execute_fn(self) -> ExecuteFn {
        Box::new(move |machine: &mut dyn Machine, bytes: &str| {
            let machine_type_name = any::type_name_of_val(machine);
            let any: &mut dyn Any = machine;

            let machine = any
                .downcast_mut::<M>()
                .ok_or(ExecuteError::UnexpectedMachineType {
                    expected: any::type_name::<M>(),
                    received: machine_type_name,
                })?;

            // Validate that the caller sent an empty argument payload
            let args: A = serde_json::from_str(bytes).map_err(ExecuteError::ParsingError)?;

            self(machine, args).map_err(ExecuteError::ExecutionError)
        })
    }
}

// --- errors ---
#[derive(Debug, Error)]
pub enum ExecuteError {
    #[error("unexpected machine type: expected `{expected}`, received `{received}`")]
    UnexpectedMachineType {
        expected: &'static str,
        received: &'static str,
    },

    #[error("command is disabled")]
    Disabled,

    #[error("command not found")]
    NotFound,

    #[error("failed to parse command arguments: {0}")]
    ParsingError(#[from] serde_json::Error),

    #[error("command execution failed: {0}")]
    ExecutionError(String),
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

        struct TestMachine;

        impl Machine for TestMachine {
            fn act(&mut self) -> ActResult {
                Ok(())
            }
        }

        impl TestMachine {
            fn simple_command(&mut self) -> Result<(), String> {
                println!("Hello World!");
                Ok(())
            }

            fn complex_command(&mut self, args: ComplexCommandArgs) -> Result<(), String> {
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

        let mut r = Manager::new();

        let cmd: fn(&mut TestMachine) -> Result<(), String> = TestMachine::simple_command;
        let execute = cmd.into_execute_fn();

        // --- simple ---
        let mut handle = r.register(ident, "simple", false, execute)?;
        handle.set_enabled(false);
        handle.set_enabled(true);

        // --- complex ---
        let cmd: fn(&mut TestMachine, ComplexCommandArgs) -> Result<(), String> =
            TestMachine::complex_command;
        let execute = cmd.into_execute_fn();

        let mut handle = r.register(ident, "not.simple", false, execute)?;
        handle.set_enabled(false);
        handle.set_enabled(true);

        // --- execute simple ---
        r.invoke(ident, &mut TestMachine, "simple", "null")?;

        // --- execute complex ---
        let args = ComplexCommandArgs {
            a: 2.0,
            b: 5,
            c: false,
            d: "Hello World".to_string(),
        };
        let args = &serde_json::to_string(&args)?;
        r.invoke(ident, &mut TestMachine, "not.simple", args)?;

        Ok(())
    }
}
