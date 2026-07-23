use std::{any::TypeId, collections::HashMap, marker::PhantomData, mem::MaybeUninit, ptr::NonNull};
use qitech_framework_common::MachineIdentificationUnique;

use super::kind_t;
use super::error::{
    RegisterResult, RegisterError, RegisterErrorKind, 
    ResolveResult, ResolveError, ResolveErrorKind, 
    ReadResult, ReadError,
};

pub type Extract<T> = unsafe fn(*const u8) -> T;

#[derive(Debug)]
pub struct PropertyRegistry<
    const SLOT_SIZE: usize,
    const MAX_ITEMS: usize,
    K: kind_t,
    Format,
    Metadata = (),
> {
    lookup: HashMap<Key, Entry<Format, Metadata>>,
    occupied: heapless::Vec<bool, MAX_ITEMS>,
    buf_generations: [MaybeUninit<u64>; MAX_ITEMS],
    buf_storage: [MaybeUninit<Storage<SLOT_SIZE>>; MAX_ITEMS],
    _marker: PhantomData<K>,
}

impl<const SLOT_SIZE: usize, const MAX_ITEMS: usize, K: kind_t, Format, Metadata>
    Default for PropertyRegistry<SLOT_SIZE, MAX_ITEMS, K, Format, Metadata>
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

impl<const SLOT_SIZE: usize, const MAX_ITEMS: usize, K: kind_t, Format, Metadata>
    PropertyRegistry<SLOT_SIZE, MAX_ITEMS, K, Format, Metadata>
{
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn register<T: 'static>(
        &mut self,
        ident: MachineIdentificationUnique,
        path: &'static str,
        postfix: &'static str,
        convert: Extract<Format>,
        metadata: Metadata,
    ) -> RegisterResult<PropertyHandle<T>> {
        const {
            assert!(size_of::<T>() <= size_of::<Storage<SLOT_SIZE>>());
            assert!(align_of::<T>() <= align_of::<Storage<SLOT_SIZE>>());
        }

        let index = self.find_free_slot(path)?;

        self.occupied[index] = true;
        self.buf_storage[index].write(Storage {
            bytes: [0u8; SLOT_SIZE],
        });

        let key = Key { ident, path, postfix };

        let entry = Entry {
            index,
            extract: convert,
            type_id: TypeId::of::<T>(),
            metadata,
        };

        self.lookup.insert(key, entry);

        let p_value = unsafe {
            NonNull::new_unchecked(
                self.buf_storage[index]
                    .as_mut_ptr()
                    .cast::<T>(),
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

    fn find_free_slot(&self, resource_path: &'static str) -> RegisterResult<usize> {
        self.occupied
            .iter()
            .position(|slot| !slot)
            .ok_or(RegisterError {
                resource_kind: K::KIND,
                resource_path,
                kind: RegisterErrorKind::RegistryFull,
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    ident: MachineIdentificationUnique,
    path: &'static str,
    postfix: &'static str,
}

#[derive(Debug)]
struct Entry<ExportFormat, Metadata> {
    index: usize,
    type_id: TypeId,
    extract: Extract<ExportFormat>,
    metadata: Metadata,
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
    K: kind_t,
    Format,
    Metadata = (),
> {
    registry: &'a PropertyRegistry<SLOT_SIZE, MAX_ITEMS, K, Format, Metadata>,
}

impl<'a, const SLOT_SIZE: usize, const MAX_ITEMS: usize, K: kind_t, Format, Metadata>
    PropertyReader<'a, SLOT_SIZE, MAX_ITEMS, K, Format, Metadata>
{
    pub fn new(
        registry: &'a PropertyRegistry<SLOT_SIZE, MAX_ITEMS, K, Format, Metadata>,
    ) -> Self {
        Self { registry }
    }

    pub fn read<T>(
        &self,
        handle: &PropertyAccessHandle<K, T>,
    ) -> ReadResult<&T> {
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
            Ok(&*(storage.bytes.as_ptr() as *const T))
        }
    }
}

pub struct PropertyAccessHandle<K: kind_t, T> {
    generation: u64,
    index: usize,
    _kind: PhantomData<K>,
    _type: PhantomData<T>,
}

pub struct PropertyResolver<
    'a,
    const SLOT_SIZE: usize,
    const MAX_ITEMS: usize,
    K: kind_t,
    Format,
    Metadata = (),
> {
    registry: &'a PropertyRegistry<SLOT_SIZE, MAX_ITEMS, K, Format, Metadata>,
    ident: MachineIdentificationUnique,
}

impl<'a, const SLOT_SIZE: usize, const MAX_ITEMS: usize, K: kind_t, Format, Metadata>
    PropertyResolver<'a, SLOT_SIZE, MAX_ITEMS, K, Format, Metadata>
{
    pub fn resolve<T>(
        &self,
        path: &'static str,
    ) -> ResolveResult<PropertyAccessHandle<K, T>>
    where
        T: 'static,
    {
        let key = Key { ident: self.ident, path, postfix: "" };

        let Some(Entry {
            index,
            type_id: spec_type_id,
            ..
        }) = self.registry.lookup.get(&key)
        else {
            return Err(ResolveError {
                resource_kind: K::KIND,
                resource_path: path,
                kind: ResolveErrorKind::NoSuchProperty,
            });
        };

        if *spec_type_id != TypeId::of::<T>() {
            return Err(ResolveError {
                resource_kind: K::KIND,
                resource_path: path,
                kind: ResolveErrorKind::InvalidType,
            });
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
