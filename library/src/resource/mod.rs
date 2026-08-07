use core::fmt;
use std::any::TypeId;
use std::borrow::Cow;

use qitech_framework_core::ident::MachineIdentificationUnique;

pub mod error;

mod erased;
pub use erased::Erased;

mod journal;
pub(crate) use journal::JournalHandle;
pub(crate) use journal::Journals;

pub mod conversion;

mod constraints;
pub use constraints::EnumConstraints;
pub use constraints::NumericConstraints;

mod property_view;
pub use property_view::CachedPropertyView;

mod config_property;
pub use config_property::ConfigProperty;
pub use config_property::ConfigPropertyRegistry;
pub use config_property::ConfigPropertyRegistryRegisterHandle;
pub use config_property::ConfigPropertyState;

mod state_property;
pub use state_property::StateProperty;
pub use state_property::StatePropertyRegistry;
pub use state_property::StatePropertyRegistryRegisterHandle;

mod measurements;
pub use measurements::Measurement;
pub use measurements::MeasurementRegistry;
pub use measurements::MeasurementRegistryRegisterHandle;

use crate::machine::Machine;

mod bump_allocator;
pub use bump_allocator::BumpAllocator;
pub use bump_allocator::BumpAllocatorMark;

pub struct Resources {
    pub config_properties: ConfigPropertyRegistry,
    pub state_properties: StatePropertyRegistry,
    pub measurements: MeasurementRegistry,
}

// mod command;
// pub use command::CommandDefinition;
// pub use command::CommandRegistry;

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
    type_name: &'static str,
    ident: MachineIdentificationUnique,
    path: &'static str,
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
/// Safety mechanism for ensuring invalid handles cannot accidentaly write
#[derive(Debug, Clone, Copy, Default)]
pub struct SlotInfo {
    state: SlotState,
    generation: u64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SubscriptionToken;
