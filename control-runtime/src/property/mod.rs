use std::{any::TypeId, collections::HashMap, marker::PhantomData, mem::MaybeUninit, ptr::NonNull};
use control_core::MachineIdentificationUnique;

mod types;
pub use types::*;

mod config;

type SlotIndex = usize;

// --- registry ---
pub struct PropertyRegistry<const REGISTRY_ID: usize, const MAX_ITEMS: usize> {
    lookup: HashMap<PropertyKey, SlotIndex>,
    slots: heapless::Vec<Option<MachineIdentificationUnique>, MAX_ITEMS>,
    buf_generation: [MaybeUninit<u64>; MAX_ITEMS],
    buf_type_id:    [MaybeUninit<TypeId>; MAX_ITEMS],
    buf_storage:    [MaybeUninit<PropertyStorage>; MAX_ITEMS],
}

impl<const REGISTRY_ID: usize, const MAX_ITEMS: usize> PropertyRegistry<REGISTRY_ID, MAX_ITEMS> {
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

        self.buf_generation[index].write(0);
        self.buf_type_id[index].write(TypeId::of::<T>());
        self.buf_storage[index].write(PropertyStorage { bytes: [0u8; 16] });

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

    pub(crate) fn free(&mut self, owner: MachineIdentificationUnique) {
        for index in 0..MAX_ITEMS {
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

// --- reader ---
pub struct PropertyReader<'a, const REGISTRY_ID: usize, const MAX_ITEMS: usize> {
    registry: &'a PropertyRegistry<REGISTRY_ID, MAX_ITEMS>,
}

impl<'a, const REGISTRY_ID: usize, const MAX_ITEMS: usize> 
    PropertyReader<'a, REGISTRY_ID, MAX_ITEMS>
{
    pub(crate) fn new(registry: &'a PropertyRegistry<REGISTRY_ID, MAX_ITEMS>) -> Self {
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

// --- resolver ---
pub struct PropertyResolver<'a, const REGISTRY_ID: usize, const MAX_ITEMS: usize> {
    registry: &'a PropertyRegistry<REGISTRY_ID, MAX_ITEMS>,
}

impl<'a, const REGISTRY_ID: usize, const MAX_ITEMS: usize>
    PropertyResolver<'a, REGISTRY_ID, MAX_ITEMS>
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
