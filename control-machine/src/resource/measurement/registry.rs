use std::{any::TypeId, marker::PhantomData, mem::MaybeUninit, ptr::NonNull};
use control_core::MachineIdentificationUnique;
use crate::conversion::{Wrapped, WrappedTryFromOptionalF64};
use crate::resource::{ReadError, RegisterError, ResolveError};
use super::{Measurement, Handle, Statistics, Config};

/// > Note: must use fixed sized storage since we use pointers and 
/// > otherwise a resize would invalidate all pointers
#[derive(Debug, Clone)]
pub struct MeasurementRegistry<const MAX_ITEMS: usize> {
    lookup: heapless::FnvIndexMap<Key, Entry, MAX_ITEMS>,

    // tracks which slots have valid data
    occupied: heapless::Vec<bool, MAX_ITEMS>,
    buf_generations: [MaybeUninit<u64>; MAX_ITEMS],
    buf_values: [MaybeUninit<f64>; MAX_ITEMS],
    buf_nulls:  [MaybeUninit<bool>; MAX_ITEMS],

    // list of stat entries
    stat_list: heapless::Vec<usize, MAX_ITEMS>,
}

impl<const MAX_ITEMS: usize> MeasurementRegistry<MAX_ITEMS> {
    pub fn new() -> Self {
        Self {
            lookup: Default::default(),
            occupied: Default::default(),
            buf_generations: [MaybeUninit::uninit(); MAX_ITEMS],
            buf_values: [MaybeUninit::uninit(); MAX_ITEMS],
            buf_nulls:  [MaybeUninit::uninit(); MAX_ITEMS],
            stat_list: Default::default(),
        }
    }

    pub fn register<T>(
        &mut self,
        ident: MachineIdentificationUnique,
        name: &'static str,
        config: Config,
        initial_value: T::Inner,
    ) -> Result<Measurement<T>, RegisterError> 
    where 
        T: Wrapped + 'static,
    {
        let key = Key { ident, name };

        if self.lookup.contains_key(&key) {
            return Err(RegisterError::AlreadyRegistered { name });
        }

        let index = self.claim_slot(name)?;
        let entry = Entry { index, type_id: TypeId::of::<T>() };

        self.lookup.insert(key, entry)
            .expect("was able to claim slot and shares capacity");

        let handle = self.alloc::<T>(ident, name, false)?;

        // --- init stats ---
        let min = if config.record_min {
            Some(self.alloc::<T>(ident, name, true)?)
        } else { None };

        let max = if config.record_max {
            Some(self.alloc::<T>(ident, name, true)?)
        } else { None };

        Ok(Measurement {
            handle,
            stats: Statistics { min, max },
            value: initial_value,
        })
    }

    /// Release all previously-registered slots belonging to `ident`
    /// Returns the number of slots freed.
    pub fn unregister_machine(&mut self, ident: MachineIdentificationUnique) -> usize {
        // Collect keys to remove
        let mut to_remove: heapless::Vec<Key, MAX_ITEMS> = heapless::Vec::new();
        for key in self.lookup.keys() {
            if key.ident == ident {
                to_remove.push(*key).expect("Cannot overflow");
            }
        }

        for key in &to_remove {
            if let Some(entry) = self.lookup.remove(key) {
                // mark unoccupied
                self.occupied[entry.index] = false;

                // ensure old handles don't read anymore 
                unsafe {
                    *self.buf_generations[entry.index].assume_init_mut() += 1;
                }

                // remove from reset list if it was contained
                self.stat_list.retain(|&i| i != entry.index);
            }
        }

        to_remove.len()
    }
}


// --- utils ---
impl<const MAX_ITEMS: usize> MeasurementRegistry<MAX_ITEMS> {
    fn alloc<T>(
        &mut self,
        ident: MachineIdentificationUnique,
        name: &'static str,
        is_stat: bool,
    ) -> Result<Handle, RegisterError> 
    where 
        T: Wrapped + 'static,
    {
        let key = Key { ident, name };

        if self.lookup.contains_key(&key) {
            return Err(RegisterError::AlreadyRegistered { name });
        }

        let index = self.claim_slot(name)?;
        let entry = Entry { index, type_id: TypeId::of::<T>() };

        self.lookup.insert(key, entry)
            .expect("was able to claim slot and shares capacity");

        unsafe {
            let p_value = NonNull::new_unchecked(self.buf_values[index].as_mut_ptr());
            let p_null = NonNull::new_unchecked(self.buf_nulls[index].as_mut_ptr());
            if is_stat {
                self.stat_list.push(index).expect("Cannot overflow");
            }
            Ok(Handle { p_value, p_null })
        }
    }

    /// Find a free slot, reusing a previously-freed one if available,
    /// otherwise growing into unused capacity.
    fn claim_slot(&mut self, name: &'static str) -> Result<usize, RegisterError> {
        if let Some(index) = self.occupied.iter().position(|occupied| !occupied) {
            self.occupied[index] = true;
            return Ok(index);
        }

        if self.occupied.len() < MAX_ITEMS {
            let index = self.occupied.len();
            self.occupied
                .push(true)
                .map_err(|_| RegisterError::RegistryFull { name })?;
            return Ok(index);
        }

        Err(RegisterError::RegistryFull { name })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    ident: MachineIdentificationUnique,
    name: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct Entry {
    index: usize,
    type_id: TypeId,
}

// --- resolver ---
pub struct MeasurementResolver<'a, const MAX_ITEMS: usize> {
    registry: &'a MeasurementRegistry<MAX_ITEMS>,
    ident: MachineIdentificationUnique,
}

impl<'a, const MAX_ITEMS: usize> MeasurementResolver<'a, MAX_ITEMS> {
    pub fn resolve<T: 'static>(
        &self,
        name: &'static str,
    ) -> Result<ReaderHandle<T>, ResolveError> {
        let key = Key { ident: self.ident, name };

        let Some(Entry { index, type_id }) = self.registry.lookup.get(&key) else {
            return Err(ResolveError::NoSuchProperty)
        };
        
        if *type_id != TypeId::of::<T>() {
            return Err(ResolveError::InvalidType);
        }

        let generation = unsafe {
            self.registry.buf_generations[*index].assume_init()
        };

        Ok(ReaderHandle {
            generation,
            index: *index,
            _marker: PhantomData,
        })
    }
}

// --- reader ---
#[derive(Debug)]
pub struct ReaderHandle<T> {
    generation: u64,
    index: usize,
    _marker: PhantomData<T>,
}

pub struct MeasurementReader<'a, const MAX_ITEMS: usize> {
    registry: &'a MeasurementRegistry<MAX_ITEMS>,
}

impl<'a, const MAX_ITEMS: usize> MeasurementReader<'a, MAX_ITEMS> {
    pub(crate) fn new(registry: &'a MeasurementRegistry<MAX_ITEMS>) -> Self {
        Self { registry }
    }

    pub fn read<T: WrappedTryFromOptionalF64>(
        &self,
        handle: &ReaderHandle<T>,
    ) -> Result<T::Inner, ReadError> {
        let generation = self.registry.buf_generations[handle.index];

        // Safety:
        // - register() verified T fits in storage
        // - register() stored TypeId::of::<T>()
        // - resolve() only creates PropertyHandle<T> after type check
        unsafe {
            if generation.assume_init() != handle.generation {
                return Err(ReadError);
            }

            let null = self.registry.buf_nulls[handle.index].assume_init_read();

            let value = if !null {
                Some(self.registry.buf_values[handle.index].assume_init_read())
            } else { None };
            
            let value = T::try_from_opt_f64(value).expect("T not allow to be None, found None!");
            Ok(value)
        }
    }
}
