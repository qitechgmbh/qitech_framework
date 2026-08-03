use std::any;
use std::any::Any;
use std::borrow::Cow;
use std::collections::HashMap;

use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::report::MachineCommandInvokeError;
use qitech_framework_core::report::MachineCommandInvokeTrace;

use crate::machine::Machine;
use crate::machine::resource::Journal;
use crate::machine::resource::Key;
use crate::machine::resource::error::RegisterError;
use crate::machine::resource::error::RegisterResult;
use crate::machine::resource::error::ResourceAccessError;

// --- resource managment ---
pub struct Manager {
    registry: HashMap<Key<'static>, Entry>,
    journal: Journal<MachineCommandInvokeTrace>,
}

impl Manager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &mut self,
        ident: MachineIdentificationUnique,
        path: &'static str,
        can_execute: CanExecuteFn,
        execute: ExecuteFn,
    ) -> RegisterResult<()> {
        let key = Key::from_str(ident, path);

        if self.registry.contains_key(&key) {
            panic!("AH SIKTIR");
            return Err(RegisterError::Duplicate);
        }

        self.registry.insert(
            key,
            Entry {
                can_execute,
                execute,
            },
        );
        Ok(())
    }

    pub fn unregister_machine(&mut self, ident: MachineIdentificationUnique) {
        self.registry.retain(|k, _| k.ident != ident);
    }

    pub fn can_invoke(
        &self,
        machine: &dyn Machine,
        target: MachineIdentificationUnique,
        path: &str,
    ) -> Result<bool, ResourceAccessError> {
        let key = Key {
            ident: target,
            path: Cow::Owned(path.to_string()),
        };

        let Some(Entry { can_execute, .. }) = self.registry.get(&key) else {
            return Err(ResourceAccessError::NoSuchResource);
        };

        (can_execute)(machine)
    }

    pub fn invoke(
        &mut self,
        target: MachineIdentificationUnique,
        machine: &mut dyn Machine,
        path: &str,
    ) -> Result<(), MachineCommandInvokeError> {
        let key = Key {
            ident: target,
            path: Cow::Owned(path.to_string()),
        };

        let Some(Entry {
            can_execute,
            execute,
        }) = self.registry.get(&key)
        else {
            return Err(MachineCommandInvokeError::NotFound);
        };

        let can_execute = match (can_execute)(machine) {
            Ok(v) => v,
            Err(ResourceAccessError::MachineTypeMismatch) => todo!(),
            Err(ResourceAccessError::NoSuchResource) => todo!(),
            Err(ResourceAccessError::NoSuchMachine) => todo!(),
        };

        if !can_execute {
            return Err(MachineCommandInvokeError::Disabled);
        }

        (execute)(machine)
    }

    pub fn drain_journal(&mut self, f: impl FnMut(MachineCommandInvokeTrace)) {
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
    can_execute: CanExecuteFn,
    execute: ExecuteFn,
}

// --- can execute fn ---
pub type CanExecuteFn = Box<dyn Fn(&dyn Machine) -> Result<bool, ResourceAccessError>>;

pub trait IntoCanExecuteFn {
    fn into_can_execute_fn(self) -> CanExecuteFn;
}

impl<M> IntoCanExecuteFn for fn(&M) -> bool
where
    M: Machine + 'static,
{
    fn into_can_execute_fn(self) -> CanExecuteFn {
        Box::new(move |machine: &dyn Machine| {
            let any: &dyn Any = machine;
            let machine = any
                .downcast_ref::<M>()
                .ok_or(ResourceAccessError::NoSuchMachine)?;

            Ok(self(machine))
        })
    }
}

// --- execute fn ---
pub type ExecuteFn = Box<dyn Fn(&mut dyn Machine) -> Result<(), MachineCommandInvokeError>>;

pub trait IntoExecuteFn {
    fn into_execute_fn(self) -> ExecuteFn;
}

impl<M> IntoExecuteFn for fn(&mut M) -> Result<(), String>
where
    M: Machine + 'static,
{
    fn into_execute_fn(self) -> ExecuteFn {
        Box::new(move |machine: &mut dyn Machine| {
            let machine_type_name = any::type_name_of_val(machine);
            let any: &mut dyn Any = machine;

            let machine =
                any.downcast_mut::<M>()
                    .ok_or(MachineCommandInvokeError::MachineTypeMismatch {
                        expected: any::type_name::<M>().to_string(),
                        received: machine_type_name.to_string(),
                    })?;

            self(machine).map_err(MachineCommandInvokeError::ExecutionError)
        })
    }
}

// --- testing ---
#[cfg(test)]
mod test {
    use qitech_framework_core::ident::MachineIdentification;

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
            fn can_execute(&self) -> bool {
                true
            }

            fn execute(&mut self) -> Result<(), String> {
                println!("Hello World!");
                Ok(())
            }
        }

        let mut r = Manager::new();

        let cmd: fn(&TestMachine) -> bool = TestMachine::can_execute;
        let can_execute = cmd.into_can_execute_fn();

        let cmd: fn(&mut TestMachine) -> Result<(), String> = TestMachine::execute;
        let execute = cmd.into_execute_fn();

        r.register(ident, "simple", can_execute, execute)?;
        r.invoke(ident, &mut TestMachine, "simple")?;
        Ok(())
    }
}
