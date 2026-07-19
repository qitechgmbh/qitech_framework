use std::{any::TypeId, collections::HashMap, marker::PhantomData, mem::MaybeUninit, ptr::NonNull};
use control_core::MachineIdentificationUnique;

type SlotIndex = usize;

pub struct PropertyRegistry<const REGISTRY_ID: usize, const MAX_SLOTS: usize> {
    lookup: HashMap<PropertyKey, SlotIndex>,
    slots: heapless::Vec<Option<MachineIdentificationUnique>, MAX_SLOTS>,
    buf_type_id:    [MaybeUninit<TypeId>; MAX_SLOTS],
    buf_generation: [MaybeUninit<u64>; MAX_SLOTS],
    buf_storage:    [MaybeUninit<PropertyStorage>; MAX_SLOTS],
}

impl<const REGISTRY_ID: usize, const MAX_SLOTS: usize> PropertyRegistry<REGISTRY_ID, MAX_SLOTS> {
    pub(crate) fn alloc<T: 'static>(
        &mut self,
        owner: MachineIdentificationUnique,
        name: &'static str,
    ) -> Result<NonNull<T>, PropertyAllocError> {
        if size_of::<T>() > size_of::<PropertyStorage>() {
            return Err(PropertyAllocError::TypeTooLarge);
        }

        if align_of::<T>() > align_of::<PropertyStorage>() {
            return Err(PropertyAllocError::InvalidAlignment);
        }

        let index = self.find_free_slot()?;

        self.slots[index] = Some(owner);

        self.buf_type_id[index].write(TypeId::of::<T>());
        self.buf_generation[index].write(0);
        self.buf_storage[index].write(PropertyStorage::new());

        let key = PropertyKey { owner, name };
        self.lookup.insert(key, index as SlotIndex);

        let ptr = unsafe {
            NonNull::new_unchecked(
                self.buf_storage[index]
                    .as_mut_ptr()
                    .cast::<T>(),
            )
        };

        Ok(ptr)
    }

    pub(crate) fn free(
        &mut self,
        owner: MachineIdentificationUnique,
    ) {
        for index in 0..MAX_SLOTS {
            if self.slots[index] != Some(owner) {
                continue;
            }

            self.slots[index] = None;

            unsafe {
                let generation = self.buf_generation[index].assume_init_mut();
                *generation += 1;
            }
        }

        self.lookup.retain(|key, _| {
            key.owner != owner
        });
    }

    fn find_free_slot(&self) -> Result<usize, PropertyAllocError> {
        self.slots
            .iter()
            .position(|slot| slot.is_none())
            .ok_or(PropertyAllocError::OutOfMemory)
    }
}

#[derive(Hash, PartialEq, Eq)]
struct PropertyKey {
    owner: MachineIdentificationUnique,
    name: &'static str,
}

pub struct PropertySlot {
    /// Concrete Rust type stored in `storage`.
    type_id: TypeId,

    /// Incremented whenever the slot is invalidated/reused.
    generation: u64,

    /// In-place storage for the value.
    storage: PropertyStorage,
}

#[repr(align(16))]
pub struct PropertyStorage {
    bytes: [u8; 16],
}


pub struct PropertyReader<'a, const REGISTRY_ID: usize, const MAX_SLOTS: usize> {
    registry: &'a PropertyRegistry<REGISTRY_ID, MAX_SLOTS>,
}

impl<'a, const REGISTRY_ID: usize, const MAX_SLOTS: usize> 
    PropertyReader<'a, REGISTRY_ID, MAX_SLOTS>
{
    pub(crate) fn new(registry: &'a PropertyRegistry<REGISTRY_ID, MAX_SLOTS>) -> Self {
        Self { registry }
    }

    pub fn read<T>(
        &self,
        handle: PropertyHandle<REGISTRY_ID, T>,
    ) -> Result<&T, PropertyReadError> {
        let generation = &self.registry.buf_generation[handle.slot_index];

        // Safety:
        // - register() verified T fits in storage
        // - register() stored TypeId::of::<T>()
        // - resolve() only creates PropertyHandle<T> after type check
        unsafe {
            if generation.assume_init() != handle.generation {
                return Err(PropertyReadError);
            }

            let storage = &self.registry.buf_storage[handle.slot_index].assume_init_read();
            Ok(&*(storage.bytes.as_ptr() as *const T))
        }
    }
}

pub struct PropertyHandle<const REGISTRY_ID: usize, T> {
    slot_index: usize,
    generation: u64,
    _marker: PhantomData<T>,
}

pub struct PropertyResolver<'a, const REGISTRY_ID: usize, const MAX_SLOTS: usize> {
    registry: &'a PropertyRegistry<REGISTRY_ID, MAX_SLOTS>,
}

impl<'a, const REGISTRY_ID: usize, const MAX_SLOTS: usize>
    PropertyResolver<'a, REGISTRY_ID, MAX_SLOTS>
{
    pub fn resolve<T: 'static>(
        &self,
        key: PropertyKey,
    ) -> Result<PropertyHandle<REGISTRY_ID, T>, PropertyResolveError> {
        let Some(&slot_index) = self.registry.lookup.get(&key) else {
            return Err(PropertyResolveError::NoSuchProperty)
        };
        
        let type_id = unsafe {
            self.registry.buf_type_id[slot_index].assume_init()
        };

        if type_id != TypeId::of::<T>() {
            return Err(PropertyResolveError::NoSuchProperty);
        }

        let generation = unsafe {
            self.registry.buf_generation[slot_index].assume_init()
        };

        Ok(PropertyHandle {
            slot_index,
            generation,
            _marker: PhantomData,
        })
    }
}

/// handle is stale
pub enum PropertyAllocError {
    OutOfMemory,
    TypeTooLarge,
    InvalidAlignment,
}

/// handle is stale
pub struct PropertyReadError;

pub enum PropertyResolveError {
    NoSuchProperty,
    InvaldType,
}
