use std::{any::{TypeId, type_name}, collections::HashMap, marker::PhantomData, mem::MaybeUninit, ptr::NonNull};
use control_core::MachineIdentificationUnique;

use crate::{conversion::WrappedTryFromOptionalF64, resource::{ResourceReadError, ResourceRegisterError, ResourceResolveError}};

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

#[derive(Debug)]
pub struct Registry<const REGISTRY_ID: usize, const MAX_ITEMS: usize> {
    lookup: HashMap<Key, Entry>,
    occupied: heapless::Vec<bool, MAX_ITEMS>,
    buf_generations: [MaybeUninit<u64>; MAX_ITEMS],
    buf_storage:     [MaybeUninit<Storage>; MAX_ITEMS],
}

impl<const REGISTRY_ID: usize, const MAX_ITEMS: usize> Registry<REGISTRY_ID, MAX_ITEMS> {
    pub(super) fn new() -> Self {
        Self { 
            lookup: Default::default(),
            occupied: Default::default(),
            buf_generations: [MaybeUninit::uninit(); MAX_ITEMS], 
            buf_storage: [MaybeUninit::uninit(); MAX_ITEMS],
        }
    }

    pub(crate) fn register<T: 'static>(
        &mut self,
        ident: MachineIdentificationUnique,
        name: &'static str,
    ) -> Result<Handle<T>, ResourceRegisterError> {
        if size_of::<T>() > size_of::<Storage>() {
            return Err(ResourceRegisterError::TypeTooLarge {
                r#type: type_name::<T>(),
                name,
            });
        }

        if align_of::<T>() > align_of::<Storage>() {
            return Err(ResourceRegisterError::AlignmentTooLarge {
                r#type: type_name::<T>(),
                name,
            });
        }

        let index = self.find_free_slot()?;

        self.occupied[index] = true;
        self.buf_storage[index].write(Storage { bytes: [0u8; 16] });

        let key = Key { ident, name };
        let entry = Entry { index, type_id: TypeId::of::<T>() };

        self.lookup.insert(key, entry);

        let p_value = unsafe {
            NonNull::new_unchecked(
                self.buf_storage[index]
                    .as_mut_ptr()
                    .cast::<T>(),
            )
        };

        Ok(Handle { p_value })
    }

    pub(crate) fn unregister_machine(
        &mut self,
        ident: MachineIdentificationUnique,
    ) -> usize {
        // Collect keys to remove
        let mut to_remove = heapless::Vec::<Key, MAX_ITEMS>::new();
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
            }
        }

        to_remove.len()
    }

    fn find_free_slot(&self) -> Result<usize, ResourceRegisterError> {
        self.occupied
            .iter()
            .position(|slot| !slot)
            .ok_or(ResourceRegisterError::RegistryFull { name: "TODO" })
    }
}

#[derive(Debug)]
pub struct Handle<T> {
    p_value: NonNull<T>,
}

impl<T> Handle<T> {
    pub fn read(&self) -> &T {
        unsafe { self.p_value.as_ref() }
    }

    pub fn write(&mut self, value: T) {
        unsafe { self.p_value.write(value) }
    }
}

#[derive(Clone, Copy)]
#[repr(align(16))]
pub struct Storage { bytes: [u8; 16] }

pub struct Reader<'a, const REGISTRY_ID: usize, const MAX_ITEMS: usize> {
    registry: &'a Registry<REGISTRY_ID, MAX_ITEMS>,
}

impl<'a, const REGISTRY_ID: usize, const MAX_ITEMS: usize> 
    Reader<'a, REGISTRY_ID, MAX_ITEMS>
{
    pub(crate) fn new(registry: &'a Registry<REGISTRY_ID, MAX_ITEMS>) -> Self {
        Self { registry }
    }

    pub fn read<T: WrappedTryFromOptionalF64>(
        &self,
        handle: &ReaderHandle<REGISTRY_ID, T>,
    ) -> Result<&T::Inner, ResourceReadError> {
        let generation = &self.registry.buf_generations[handle.index];

        // Safety:
        // - register() verified T fits in storage
        // - register() stored TypeId::of::<T>()
        // - resolve() only creates PropertyHandle<T> after type check
        unsafe {
            if generation.assume_init() != handle.generation {
                return Err(ResourceReadError);
            }

            let storage = &self.registry.buf_storage[handle.index].assume_init_read();
            Ok(&*(storage.bytes.as_ptr() as *const T::Inner))
        }
    }
}

pub struct ReaderHandle<const REGISTRY_ID: usize, T> {
    generation: u64,
    index: usize,
    _marker: PhantomData<T>,
}

pub struct Resolver<'a, const REGISTRY_ID: usize, const MAX_ITEMS: usize> {
    registry: &'a Registry<REGISTRY_ID, MAX_ITEMS>,
    ident: MachineIdentificationUnique, 
}

impl<'a, const REGISTRY_ID: usize, const MAX_ITEMS: usize>
    Resolver<'a, REGISTRY_ID, MAX_ITEMS>
{
    pub fn resolve<T: 'static>(
        &self,
        name: &'static str,
    ) -> Result<ReaderHandle<REGISTRY_ID, T>, ResourceResolveError> {
        let key = Key { ident: self.ident, name };

        let Some(Entry { index, type_id }) = self.registry.lookup.get(&key) else {
            return Err(ResourceResolveError::NoSuchProperty)
        };
        
        if *type_id != TypeId::of::<T>() {
            return Err(ResourceResolveError::NoSuchProperty);
        }

        let generation = unsafe {
            self.registry.buf_generations[*index].assume_init()
        };

        Ok(ReaderHandle {
            index: *index,
            generation,
            _marker: PhantomData,
        })
    }
}
