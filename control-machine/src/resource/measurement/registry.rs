use std::{any::TypeId, mem::MaybeUninit, ptr::NonNull};
use control_core::MachineIdentificationUnique;
use crate::{conversion::Wrapped, resource::RegisterError};
use super::Measurement;

/// > Note: must use fixed sized storage since we use pointers and 
/// > otherwise a resize would invalidate all pointers
#[derive(Debug, Clone)]
pub struct MeasurementRegistry<const REGISTRY_ID: usize, const MAX_ITEMS: usize> {
    lookup: heapless::FnvIndexMap<Key, Entry, MAX_ITEMS>, a

    // tracks which slots have valid data
    occupied: heapless::Vec<bool, MAX_ITEMS>,
    buf_generations: [MaybeUninit<u64>; MAX_ITEMS],
    buf_values: [MaybeUninit<f64>; MAX_ITEMS],
    buf_nulls:  [MaybeUninit<bool>; MAX_ITEMS],

    // list of stat entries
    stat_list: heapless::Vec<usize, MAX_ITEMS>,
}

impl<const REGISTRY_ID: usize, const MAX_ITEMS: usize> MeasurementRegistry<REGISTRY_ID, MAX_ITEMS> {
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

    fn alloc<T>(
        &mut self,
        ident: MachineIdentificationUnique,
        name: &'static str,
        is_stat: bool,
    ) -> Result<(NonNull<f64>, NonNull<bool>), RegisterError> 
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
            Ok((p_value, p_null))
        }
    }

    pub fn register<T>(
        &mut self,
        ident: MachineIdentificationUnique,
        name: &'static str,
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

        unsafe {
            let p_value = NonNull::new_unchecked(self.buf_values[index].as_mut_ptr());
            let p_null = NonNull::new_unchecked(self.buf_nulls[index].as_mut_ptr());

            Ok(Measurement {
                p_value,
                p_null,
                stats: todo!(),
                value: todo!(),
            })
        }
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