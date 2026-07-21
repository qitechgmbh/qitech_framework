use control_core::MachineIdentificationUnique;
use std::{any::TypeId, collections::HashMap, marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

use crate::resource::{ReadError, RegisterError, ResolveError};
use crate::{
    conversion::{Convertible, PropertyType},
    resource::{Kind, RegisterErrorReason},
};

type Converter<T> = unsafe fn(*const u8) -> T;

#[derive(Debug)]
pub struct PropertyRegistry<
    const SLOT_SIZE: usize,
    const MAX_ITEMS: usize,
    K: Kind,
    Format,
> {
    lookup: HashMap<Key, Entry<Format>>,
    occupied: heapless::Vec<bool, MAX_ITEMS>,
    buf_generations: [MaybeUninit<u64>; MAX_ITEMS],
    buf_storage: [MaybeUninit<Storage<SLOT_SIZE>>; MAX_ITEMS],
    _marker: PhantomData<K>,
}

impl<const SLOT_SIZE: usize, const MAX_ITEMS: usize, K: Kind, Format>
    Default for PropertyRegistry<SLOT_SIZE, MAX_ITEMS, K, Format>
{
    fn default() -> Self {
        Self {
            lookup: Default::default(),
            occupied: Default::default(),
            buf_generations: [MaybeUninit::uninit(); MAX_ITEMS],
            buf_storage: [MaybeUninit::uninit(); MAX_ITEMS],
            _marker: PhantomData,
        }
    }
}

impl<const SLOT_SIZE: usize, const MAX_ITEMS: usize, K: Kind, Format>
    PropertyRegistry<SLOT_SIZE, MAX_ITEMS, K, Format>
{
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn register<T>(
        &mut self,
        ident: MachineIdentificationUnique,
        name: &'static str,
        postfix: &'static str,
    ) -> Result<PropertyHandle<T::Value>, RegisterError>
    where
        T: PropertyType + Convertible<Format> + 'static,
    {
        const {
            assert!(size_of::<T::Value>() <= size_of::<Storage<SLOT_SIZE>>());
            assert!(align_of::<T::Value>() <= align_of::<Storage<SLOT_SIZE>>());
        }

        let index = self.find_free_slot(name)?;

        self.occupied[index] = true;
        self.buf_storage[index].write(Storage {
            bytes: [0u8; SLOT_SIZE],
        });

        let key = Key { ident, name, postfix };

        let entry = Entry {
            index,
            converter: T::convert,
            type_id: TypeId::of::<T>(),
        };

        self.lookup.insert(key, entry);

        let p_value = unsafe {
            NonNull::new_unchecked(
                self.buf_storage[index]
                    .as_mut_ptr()
                    .cast::<T::Value>(),
            )
        };

        Ok(PropertyHandle { p_value })
    }

    pub(crate) fn unregister_machine(&mut self, ident: MachineIdentificationUnique) -> usize {
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

    fn find_free_slot(&self, resource_name: &'static str) -> Result<usize, RegisterError> {
        self.occupied
            .iter()
            .position(|slot| !slot)
            .ok_or(RegisterError {
                resource_kind: K::KIND,
                resource_name,
                reason: RegisterErrorReason::RegistryFull,
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    ident: MachineIdentificationUnique,
    name: &'static str,
    postfix: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct Entry<ExportFormat> {
    index: usize,
    type_id: TypeId,
    converter: Converter<ExportFormat>,
}

#[derive(Debug)]
pub struct PropertyHandle<T> {
    p_value: NonNull<T>,
}

impl<T> PropertyHandle<T> {
    pub fn read(&self) -> &T {
        unsafe { self.p_value.as_ref() }
    }

    pub fn write(&self, value: T) {
        unsafe { self.p_value.write(value) }
    }
}

#[derive(Clone, Copy)]
#[repr(align(16))]
struct Storage<const SLOT_SIZE: usize> {
    bytes: [u8; SLOT_SIZE],
}

pub struct PropertyReader<
    'a,
    const SLOT_SIZE: usize,
    const MAX_ITEMS: usize,
    K: Kind,
    Format,
> {
    registry: &'a PropertyRegistry<SLOT_SIZE, MAX_ITEMS, K, Format>,
}

impl<'a, const SLOT_SIZE: usize, const MAX_ITEMS: usize, K: Kind, Format>
    PropertyReader<'a, SLOT_SIZE, MAX_ITEMS, K, Format>
{
    pub fn new(
        registry: &'a PropertyRegistry<SLOT_SIZE, MAX_ITEMS, K, Format>,
    ) -> Self {
        Self { registry }
    }

    pub fn read<T>(
        &self,
        handle: &PropertyAccessHandle<K, T::Value>,
    ) -> Result<&T::Value, ReadError>
    where
        T: PropertyType
    {
        let generation = &self.registry.buf_generations[handle.index];

        // Safety:
        // - register() verified T fits in storage
        // - register() stored TypeId::of::<T>()
        // - resolve() only creates PropertyHandle<T> after type check
        unsafe {
            if generation.assume_init() != handle.generation {
                return Err(ReadError);
            }

            let storage = &self.registry.buf_storage[handle.index].assume_init_read();
            Ok(&*(storage.bytes.as_ptr() as *const T::Value))
        }
    }
}

pub struct PropertyAccessHandle<K: Kind, T> {
    generation: u64,
    index: usize,
    _kind: PhantomData<K>,
    _type: PhantomData<T>,
}

pub struct PropertyResolver<
    'a,
    const SLOT_SIZE: usize,
    const MAX_ITEMS: usize,
    K: Kind,
    Format,
> {
    registry: &'a PropertyRegistry<SLOT_SIZE, MAX_ITEMS, K, Format>,
    ident: MachineIdentificationUnique,
}

impl<'a, const SLOT_SIZE: usize, const MAX_ITEMS: usize, K: Kind, Format>
    PropertyResolver<'a, SLOT_SIZE, MAX_ITEMS, K, Format>
{
    pub fn resolve<T>(
        &self,
        name: &'static str,
    ) -> Result<PropertyAccessHandle<K, T::Value>, ResolveError>
    where
        T: PropertyType + 'static,
    {
        let key = Key { ident: self.ident, name, postfix: "" };

        let Some(Entry {
            index,
            type_id: spec_type_id,
            ..
        }) = self.registry.lookup.get(&key)
        else {
            return Err(ResolveError::NoSuchProperty);
        };

        if *spec_type_id != TypeId::of::<T>() {
            return Err(ResolveError::InvalidType);
        }

        let generation = unsafe { self.registry.buf_generations[*index].assume_init() };

        Ok(PropertyAccessHandle {
            index: *index,
            generation,
            _type: PhantomData,
            _kind: PhantomData,
        })
    }
}
