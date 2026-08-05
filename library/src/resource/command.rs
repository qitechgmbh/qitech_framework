use std::collections::HashMap;
use std::mem::transmute;
use std::ptr::NonNull;

use qitech_framework_core::ident::MachineIdentificationUnique;

use crate::machine::Machine;
use crate::resource::Key;
use crate::resource::error::RegisterError;
use crate::resource::error::RegisterResult;

pub struct CommandRegistry {
    commands: Vec<ExecuteContext>,
    entries: HashMap<Key<'static>, usize>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &mut self,
        ident: MachineIdentificationUnique,
        path: &'static str,
        entry: ExecuteContext,
    ) -> RegisterResult<()> {
        let key = Key::from_str(ident, path);

        if self.entries.contains_key(&key) {
            return Err(RegisterError::Duplicate);
        }

        self.commands.push(entry);

        self.entries.insert(key, self.commands.len() - 1);

        Ok(())
    }

    pub fn unregister_machine(&mut self, ident: MachineIdentificationUnique) {
        let mut remove_indices = self
            .entries
            .iter()
            .filter_map(|(key, index)| (key.ident == ident).then_some(*index))
            .collect::<Vec<_>>();

        if remove_indices.is_empty() {
            return;
        }

        remove_indices.sort_unstable();

        // Remove from highest index first so Vec indices stay valid
        for index in remove_indices.iter().rev() {
            self.commands.swap_remove(*index);
        }

        // Rebuild index map
        self.entries.retain(|_, index| *index < self.commands.len());

        // Fix indices after swap_remove
        for index in self.entries.values_mut() {
            if *index >= self.commands.len() {
                continue;
            }
        }
    }

    pub fn get(&self, key: Key) -> Option<&ExecuteContext> {
        let Some(index) = self.entries.get(&key) else {
            return None;
        };

        let entry = self
            .commands
            .get(*index)
            .expect("entries and commands must be in sync");

        Some(entry)
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self {
            entries: Default::default(),
            commands: Default::default(),
        }
    }
}

pub struct ExecuteContext {
    machine: NonNull<()>,
    can_execute: fn(NonNull<()>, *const ()) -> bool,
    execute: fn(NonNull<()>, *const ()) -> Result<(), String>,

    can_execute_fn: *const (),
    execute_fn: *const (),
}

impl ExecuteContext {
    pub fn can_execute(&self) -> bool {
        (self.can_execute)(self.machine, self.can_execute_fn)
    }

    pub fn execute(&self) -> Result<(), String> {
        (self.execute)(self.machine, self.execute_fn)
    }
}

pub struct CommandDefinition {
    pub path: &'static str,
    pub can_execute: *const (),
    pub execute: *const (),
}

impl CommandDefinition {
    pub fn into_entry<M: Machine>(self, machine: NonNull<M>) -> ExecuteContext {
        ExecuteContext {
            machine: machine.cast(),
            can_execute_fn: self.can_execute,
            execute_fn: self.execute,

            can_execute: can_execute_adapter::<M>,
            execute: execute_adapter::<M>,
        }
    }
}

fn can_execute_adapter<T>(machine: NonNull<()>, function: *const ()) -> bool
where
    T: Machine + 'static,
{
    let machine = unsafe { machine.cast::<T>().as_ref() };

    let function: fn(&T) -> bool = unsafe { transmute(function) };

    function(machine)
}

fn execute_adapter<M>(machine: NonNull<()>, function: *const ()) -> Result<(), String>
where
    M: Machine + 'static,
{
    let machine = unsafe { machine.cast::<M>().as_mut() };

    let function: fn(&mut M) -> Result<(), String> = unsafe { transmute(function) };

    function(machine)
}
