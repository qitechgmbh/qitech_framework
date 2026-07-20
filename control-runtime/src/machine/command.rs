use std::{any::Any, cell::RefCell, collections::HashMap, marker::PhantomData, ptr::NonNull, rc::{Rc, Weak}};
use control_core::MachineIdentificationUnique;
use serde::de::DeserializeOwned;
use crate::MachineBuilder;

use super::build;

// command registry: (machine -> list of func + pre)

pub struct CommandEntry<M, A> {
    func: fn(&mut M, A),
    availibility: CommandAvailability,
}

impl<M, A> CommandEntry<M, A> {
    pub(crate) fn invoke(&self, machine: &mut M, args: A) {
        assert!(matches!(self.availibility, CommandAvailability::Available));
        (self.func)(machine, args);
    }

    pub fn make_available(&mut self) {
        // TODO: write into the system
        self.availibility = CommandAvailability::Available;
    }

    pub fn make_unavailable(&mut self, reason: &'static str) {
        // TODO: write into the system
        self.availibility = CommandAvailability::Unavailable { reason };
    }
}

// exposed to user
pub struct Command {
    // info for putting into report entry
    ident: MachineIdentificationUnique,
    name: &'static str,

    /// handle to update slot in the registry
    p_availibility: NonNull<CommandAvailability>,

    /// handle to append changes of availability to report
    emitter: (),
}

impl Command {
    pub fn make_available(&mut self) {
        // TODO: write into the system
    }

    pub fn make_unavailable(&mut self, reason: &'static str) {
        // TODO: write into the system
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CommandAvailability {
    Available,
    Unavailable { reason: &'static str }
}

pub struct CommandRegistry {
    inner: HashMap<MachineIdentificationUnique, HashMap<&'static str, Rc<RefCell<CommandRegistryEntry>>>>,
}

impl CommandRegistry {
    pub fn execute(
        &self,
        id: &MachineIdentificationUnique,
        name: &str,
        machine: &mut dyn Any,
        bytes: &[u8],
    ) -> Result<(), CommandError> {
        let entry = self
            .inner
            .get(id)
            .and_then(|cmds| cmds.get(name))
            .ok_or(CommandError::NotFound)?;

        let entry = entry.borrow();
        (entry.exec)(machine, bytes)
    }
}

pub enum CommandState {
    Enabled,
    Disabled { reason: &'static str }
}

pub struct CommandRegistryEntry {
    state: CommandState,
    exec: Box<dyn Fn(&mut dyn Any, &[u8]) -> Result<(), CommandError>>,
}

impl CommandRegistryEntry {
    fn new<M: 'static, A>(
        command: fn(&mut M, A), 
        predicate: Option<fn(&mut M, &A) -> bool>
    ) -> Self
    where
        A: DeserializeOwned + 'static,
    {
        Self {
            exec: Box::new(move |machine: &mut dyn Any, bytes: &[u8]| {
                let machine = machine
                    .downcast_mut::<M>()
                    .expect("command dispatched to wrong machine type");

                let args = serde_json::from_slice(bytes)
                    .map_err(|e| CommandError::Deserialize(e.to_string()))?;

                if let Some(pred) = predicate {
                    pred(machine, &args);
                }

                command(machine, args);
                Ok(())
            }),
        }
    }
}

pub enum CommandError {
    NotFound,
    Deserialize(String),
}

pub struct CommandHandle<A> {
    entry: Weak<RefCell<CommandRegistryEntry>>,
    _marker: PhantomData<A>,
}

impl<A> CommandHandle<A> {
    pub fn enable(&self) {
        let entry = self.entry.upgrade().expect("Cannot outlive runtimes rc");
        entry.borrow_mut().state = CommandState::Enabled;
    }

    pub fn disable(&self, reason: &'static str) {
        let entry = self.entry.upgrade().expect("Cannot outlive runtimes rc");
        entry.borrow_mut().state = CommandState::Disabled { reason };
    }
}
