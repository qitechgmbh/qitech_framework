use core::fmt;
use std::any::TypeId;
use std::borrow::Cow;

use qitech_framework_core::ident::MachineIdentificationUnique;

pub mod error;

mod journal;
pub(crate) use journal::Journal;
pub(crate) use journal::JournalHandle;
pub(crate) use journal::Journals;

pub mod conversion;

pub mod constraints;
mod property_view;

mod config_property;
pub use config_property::ConfigProperty;
pub use config_property::ConfigPropertyRegistry;
pub use config_property::ConfigPropertyRegistryRegisterHandle;
pub use config_property::ConfigPropertyState;

use crate::machine::Machine;

mod bump_allocator;
pub use bump_allocator::BumpAllocator;
pub use bump_allocator::BumpAllocatorMark;

// mod command;
// pub use command::CommandDefinition;
// pub use command::CommandRegistry;

// --- registry ---
pub struct Resources {
    pub config_properties: ConfigPropertyRegistry,
}

// --- machine ---
#[derive(Debug)]
pub struct MachineInfo {
    ident: MachineIdentificationUnique,
    pos: usize,
    len: usize,
}

#[derive(Clone, Copy)]
pub struct PropertyDescriptor {
    type_id: TypeId,
    ident: MachineIdentificationUnique,
    resource: &'static str,
    p_value: *mut (),
    p_cache: *mut (),
    p_state: *mut (),
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum SlotState {
    /// slot was never used before
    #[default]
    Unused,

    /// slot is reserved but net yet commited, writing data to it in this state will panic
    Reserved,

    /// slot was reserved and the commit succeeded
    Activated,

    /// slot was initialized but the machine has been lost, writing will result in a panic
    Deactivated,
}

// --- key ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key<'a> {
    ident: MachineIdentificationUnique,
    path: Cow<'a, str>,
}

impl<'a> Key<'a> {
    pub fn from_str(ident: MachineIdentificationUnique, path: &'a str) -> Self {
        Self {
            ident,
            path: Cow::Borrowed(path),
        }
    }
}

// --- kind ---
#[derive(Debug, Clone, Copy)]
pub enum ResourceKind {
    ConfigProperty,
    StateProperty,
    Measurement,
    Command,
    Event,
}

impl fmt::Display for ResourceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::ConfigProperty => "config property",
            Self::StateProperty => "state property",
            Self::Measurement => "measurement",
            Self::Command => "command",
            Self::Event => "event",
        };
        f.write_str(s)
    }
}

// --- misc ---
#[derive(Clone, Copy)]
pub struct OnExternalChangedCallback {
    pub func: *const (),
    pub adapter: fn(&mut dyn Machine, *const ()),
}

// --- utils ---
fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

// --- subscribe
