use std::{any::TypeId, fmt, marker::PhantomData};
use control_core::MachineIdentificationUnique;

pub struct PropertyHandle<const REGISTRY_ID: usize, T> {
    pub(super) slot_index: usize,
    pub(super) generation: u64,
    pub(super) _marker: PhantomData<T>,
}

#[repr(align(16))]
pub struct PropertyStorage { pub(super) bytes: [u8; 16] }

#[derive(Hash, PartialEq, Eq)]
pub struct PropertyKey {
    pub(super) owner: MachineIdentificationUnique,
    pub(super) name: &'static str,
}

pub struct PropertySlot {
    /// Concrete Rust type stored in `storage`.
    pub(super) type_id: TypeId,

    /// Incremented whenever the slot is invalidated/reused.
    pub(super) generation: u64,

    /// In-place storage for the value.
    pub(super) storage: PropertyStorage,
}

// --- errors ---
#[derive(Debug, Clone, Copy)]
pub enum PropertyAllocError {
    OutOfMemory,
    TypeTooLarge,
    InvalidAlignment,
}

impl fmt::Display for PropertyAllocError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::OutOfMemory => {
                write!(f, "property registry has no free slots")
            }
            Self::TypeTooLarge => {
                write!(f, "property type exceeds storage size")
            }
            Self::InvalidAlignment => {
                write!(f, "property type has unsupported alignment")
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PropertyResolveError {
    NoSuchProperty,
    InvalidType,
}

impl fmt::Display for PropertyResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSuchProperty => write!(f, "property does not exist"),
            Self::InvalidType => write!(f, "property type does not match requested type"),
        }
    }
}

/// handle is stale
#[derive(Debug, Clone, Copy)]
pub struct PropertyReadError;

impl fmt::Display for PropertyReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "property handle is stale")
    }
}

impl std::error::Error for PropertyAllocError {}
impl std::error::Error for PropertyReadError {}
impl std::error::Error for PropertyResolveError {}