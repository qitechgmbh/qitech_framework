use std::{marker::PhantomData, ptr::NonNull};

use control_core::MachineIdentificationUnique;


// command registry: (machine -> list of func + pre)


pub struct CommandEntry<M, A> {
    func: fn(&mut M, A),
    pred: fn(&mut M) -> bool,
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
