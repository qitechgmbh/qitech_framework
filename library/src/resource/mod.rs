mod bump_allocator;
pub(crate) use bump_allocator::BumpAllocator;
pub(crate) use bump_allocator::BumpAllocatorMark;

mod journal;
pub(crate) use journal::Journal;
pub(crate) use journal::JournalHandle;
pub(crate) use journal::Journals;

mod property_registry;
pub(crate) use property_registry::PropertyRegistrar;
pub(crate) use property_registry::PropertyRegistry;
pub(crate) use property_registry::SlotDescriptor;

pub(crate) mod constraints;