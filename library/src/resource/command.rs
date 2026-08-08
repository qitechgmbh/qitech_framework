use std::collections::HashMap;
use std::mem::transmute;
use std::ptr::NonNull;

use qitech_framework_core::ident::MachineIdentificationUnique;

use crate::machine::Machine;
use crate::resource::Key;
use crate::resource::error::RegisterError;
use crate::resource::error::RegisterResult;

pub struct CommandRegistry {
    commands: Box<[ExecuteContext]>,
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

    pub fn register_machine(
        &'_ mut self,
        ident: MachineIdentificationUnique,
    ) -> ConfigPropertyRegistryRegisterHandle<'_> {
        let item_pos = self.buf_len;
        let value_mark = self.alloc_value.mark();
        let cache_mark = self.alloc_cache.mark();
        let state_mark = self.alloc_state.mark();

        ConfigPropertyRegistryRegisterHandle {
            registry: self,
            ident,
            item_pos,
            value_mark,
            cache_mark,
            state_mark,
            committed: false,
        }
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

// --- registrar ---
pub struct CommandRegistrar<'a> {
    registry: &'a mut CommandRegistry,
    ident: MachineIdentificationUnique,
    items: Vec<CommandItem>,
}

impl<'a> CommandRegistrar<'a> {
    pub fn register<M: Machine + 'static>(
        &mut self,
        resource: &'static str,
        can_execute: Option<fn(&M) -> bool>,
        execute: fn(&mut M) -> Result<(), String>,
    ) {
        let is_duplicate = self
            .items
            .iter()
            .find(|item| item.resource == resource)
            .is_some();

        if is_duplicate {
            panic!("TODO: return register error");
        }

        self.items.push(CommandItem {
            resource,
            can_execute: (),
            execute: (),
        });
    }
}

pub struct CommandItem {
    resource: &'static str,
    can_execute: Option<Box<dyn Fn(&dyn Machine) -> bool>>,
    execute: Box<dyn Fn(&mut dyn Machine) -> Result<(), String>>,
}

pub struct Entry {
    ident: MachineIdentificationUnique,
    path: &'static str,
    exec_ctx: ExecuteContext,
}

pub struct ExecuteContext {
    execute_adapter: fn(*const (), NonNull<()>) -> bool,
    execute: fn(*const (), NonNull<()>) -> Result<(), String>,

    can_execute_adapter: *const (),
    can_execute: *const (),
}

impl ExecuteContext {
    pub fn can_execute(&self) -> bool {
        (self.execute_adapter)(self.machine, self.execute_adapter)
    }

    pub fn execute(&self) -> Result<(), String> {
        (self.execute)(self.machine, self.execute)
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
