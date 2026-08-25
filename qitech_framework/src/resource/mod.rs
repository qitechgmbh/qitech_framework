use std::rc::Rc;
use std::rc::Weak;

use qitech_framework_core::ident::MachineInstanceIdentification;

mod bump_allocator;
pub(crate) use bump_allocator::BumpAllocator;
pub(crate) use bump_allocator::BumpAllocatorMark;

mod journal;
pub(crate) use journal::JournalHandle;
pub(crate) use journal::Journals;

mod property_registry;
pub(crate) use property_registry::PropertyRegistrar;
pub(crate) use property_registry::PropertyRegistry;

pub(crate) mod constraints;
pub(crate) mod conversion;
use conversion::ReadMeasurementFn;

// --- registry ---
pub struct ResourceRegistry {
    pub config_properties: PropertyRegistry,
    pub state_properties: PropertyRegistry,
    pub measurements: PropertyRegistry<ReadMeasurementFn>,
}

// --- key ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceKey {
    pub ident: MachineInstanceIdentification,
    pub path: &'static str,
}

// --- lifetime token ---
#[derive(Debug, Default)]
pub struct LifetimeTokenOwner {
    inner: Rc<()>,
}

impl LifetimeTokenOwner {
    pub fn new() -> Self {
        Self { inner: Rc::new(()) }
    }

    pub fn new_token(&self) -> LifetimeToken {
        LifetimeToken {
            inner: Rc::downgrade(&self.inner),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LifetimeToken {
    inner: Weak<()>,
}

impl LifetimeToken {
    pub(crate) fn validate(&self) {
        assert!(!self.expired(), "LifetimeToken outlived LifetimeTokenOwner");
    }

    fn expired(&self) -> bool {
        self.inner.upgrade().is_none()
    }
}
