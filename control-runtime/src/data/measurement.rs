use std::{any::TypeId, marker::PhantomData, mem::MaybeUninit, ptr::NonNull};
use control_core::MachineIdentificationUnique;

use crate::conversion::FromF64;

#[derive(Debug, Clone, Copy, Hash)]
struct Key {
    ident: MachineIdentificationUnique,
    name: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct Entry {
    index: u16,
    type_id: TypeId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterError {
    AlreadyRegistered,
    RegistryFull,
}

/// > Note: must use fixed sized storage since we use pointers and 
/// > and resize would invalidate all pointers
#[derive(Debug, Clone)]
pub struct Registry<const REGISTRY_ID: usize, const MAX_ITEMS: usize> {
    lookup: heapless::FnvIndexMap<Key, Entry, MAX_ITEMS>,

    // tracks which slots have valid data
    occupied: heapless::Vec<bool, MAX_ITEMS>,
    buf_generations: [MaybeUninit<u64>; MAX_ITEMS],
    buf_values: [MaybeUninit<f64>; MAX_ITEMS],
    buf_nulls:  [MaybeUninit<bool>; MAX_ITEMS],

    // --- misc ---
    reset_list: heapless::Vec<usize, MAX_ITEMS>,
}

impl<const REGISTRY_ID: usize, const MAX_ITEMS: usize> Registry<REGISTRY_ID, MAX_ITEMS> {
    pub fn new() -> Self {
        Self {
            lookup: Default::default(),
            occupied: Default::default(),
            buf_generations: [MaybeUninit::uninit(); MAX_ITEMS],
            buf_values: [MaybeUninit::uninit(); MAX_ITEMS],
            buf_nulls:  [MaybeUninit::uninit(); MAX_ITEMS],
            reset_list: Default::default(),
        }
    }

    pub fn register<T: 'static>(
        &mut self,
        ident: MachineIdentificationUnique,
        name: &'static str,
    ) -> Result<WriteHandle, RegisterError> {
        let key = Key { ident, name };

        if self.lookup.contains_key(&key) {
            return Err(RegisterError::AlreadyRegistered);
        }

        let index = self.claim_slot()? as u16;
        let entry = Entry { index, type_id: TypeId::of::<T>() };

        self.lookup.insert(key, entry)
            .expect("was able to claim slot and shares capacity");

        unsafe {
            let p_value = NonNull::new_unchecked(self.buf_values[index].as_mut_ptr());
            let p_null = NonNull::new_unchecked(self.buf_nulls[index].as_mut_ptr());
            Ok(WriteHandle { p_value, p_null })
        }
    }

    /// Release all previously-registered slots belonging to `ident`
    /// Returns the number of slots freed.
    pub fn unregister(&mut self, ident: &MachineIdentificationUnique) -> usize {
        // Collect keys to remove
        let mut to_remove: heapless::Vec<Key, MAX_ITEMS> = heapless::Vec::new();
        for key in self.lookup.keys() {
            if &key.ident == ident {
                to_remove.push(key.clone()).expect("Cannot overflow");
            }
        }

        for key in &to_remove {
            if let Some(entry) = self.lookup.remove(key) {
                // mark unoccupied
                self.occupied[entry.index] = false;

                // ensure old handles don't read anymore 
                self.buf_generations[entry.index] += 1;

                // remove from reset list if it was contained
                self.reset_list.retain(|&i| i != entry.index);
            }
        }

        to_remove.len()
    }

    /// Find a free slot, reusing a previously-freed one if available,
    /// otherwise growing into unused capacity.
    fn claim_slot(&mut self) -> Result<u16, RegisterError> {
        if let Some(index) = self.occupied.iter().position(|occupied| !occupied) {
            self.occupied[index] = true;
            return Ok(index as u16);
        }

        if self.occupied.len() < MAX_ITEMS {
            let index = self.occupied.len();
            self.occupied
                .push(true)
                .map_err(|_| RegisterError::RegistryFull)?;
            return Ok(index as u16);
        }

        Err(RegisterError::RegistryFull)
    }
}

#[derive(Debug)]
pub struct WriteHandle {
    p_value: NonNull<f64>,
    p_null: NonNull<bool>,
}

impl WriteHandle {
    pub fn write(&mut self, value: Option<f64>) {
        unsafe {
            match value {
                Some(v) => {
                    self.p_value.write(v);
                    self.p_null.write(false);
                }
                None => {
                    self.p_null.write(true);
                }
            }
        }
    }
}

// --- reader ---

/// handle is stale
#[derive(Debug, Clone, Copy)]
pub struct ReadError;

#[derive(Debug)]
pub struct ReadHandle<const REGISTRY_ID: usize, T> {
    generation: u64,
    slot_index: usize,
    _marker: PhantomData<T>,
}

pub struct Reader<'a, const REGISTRY_ID: usize, const MAX_SLOTS: usize> {
    registry: &'a Registry<REGISTRY_ID, MAX_SLOTS>,
}

impl<'a, const REGISTRY_ID: usize, const MAX_SLOTS: usize> 
    Reader<'a, REGISTRY_ID, MAX_SLOTS>
{
    pub(crate) fn new(registry: &'a Registry<REGISTRY_ID, MAX_SLOTS>) -> Self {
        Self { registry }
    }

    pub fn read<T: FromF64>(
        &self,
        handle: ReadHandle<REGISTRY_ID, T>,
    ) -> Result<T::Output, ReadError> {
        let generation = self.registry.buf_generations[handle.slot_index];

        // Safety:
        // - register() verified T fits in storage
        // - register() stored TypeId::of::<T>()
        // - resolve() only creates PropertyHandle<T> after type check
        unsafe {
            if generation.assume_init() != handle.generation {
                return Err(ReadError);
            }

            let value = self.registry.buf_values[handle.slot_index].assume_init_read();
            Ok(T::from_f64(value))
        }
    }
}

// --- resolver ---
#[derive(Debug, Clone, Copy)]
pub enum ResolveError {
    NoSuchProperty,
    InvalidType,
}

pub struct Resolver<'a, const REGISTRY_ID: usize, const MAX_SLOTS: usize> {
    registry: &'a Registry<REGISTRY_ID, MAX_SLOTS>,
}

impl<'a, const REGISTRY_ID: usize, const MAX_SLOTS: usize> Resolver<'a, REGISTRY_ID, MAX_SLOTS> {
    pub fn resolve<T: 'static>(
        &self,
        ident: MachineIdentificationUnique,
        name: &'static str,
    ) -> Result<ReadHandle<REGISTRY_ID, T>, ResolveError> {
        let key = Key { ident, name };

        let Some(&slot_index) = self.registry.lookup.get(&key) else {
            return Err(ResolveError::NoSuchProperty)
        };
        
        let type_id = unsafe {
            self.registry.buf_type_id[slot_index].assume_init()
        };

        if type_id != TypeId::of::<T>() {
            return Err(ResolveError::InvalidType);
        }

        let generation = unsafe {
            self.registry.buf_generation[slot_index].assume_init()
        };

        Ok(ReadHandle {
            slot_index,
            generation,
            _marker: PhantomData,
        })
    }
}
