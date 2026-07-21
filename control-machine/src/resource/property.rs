use control_core::MachineIdentificationUnique;
use std::{any::TypeId, collections::HashMap, marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

use crate::resource::{ReadError, RegisterError, ResolveError};
use crate::{
    conversion::{Convertible, PropertyType},
    resource::{Kind, RegisterErrorReason, Specification},
};

type Converter<T> = unsafe fn(*const u8) -> T;

#[derive(Debug)]
pub struct PropertyRegistry<
    const REGISTRY_ID: usize,
    const SLOT_SIZE: usize,
    const MAX_ITEMS: usize,
    RK: Kind,
    Format,
> {
    lookup: HashMap<Key, Entry<Format>>,
    occupied: heapless::Vec<bool, MAX_ITEMS>,
    buf_generations: [MaybeUninit<u64>; MAX_ITEMS],
    buf_storage: [MaybeUninit<Storage<SLOT_SIZE>>; MAX_ITEMS],
    _marker: PhantomData<RK>,
}

impl<const REGISTRY_ID: usize, const SLOT_SIZE: usize, const MAX_ITEMS: usize, RK: Kind, Format>
    Default for PropertyRegistry<REGISTRY_ID, SLOT_SIZE, MAX_ITEMS, RK, Format>
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

impl<const REGISTRY_ID: usize, const SLOT_SIZE: usize, const MAX_ITEMS: usize, RK: Kind, Format>
    PropertyRegistry<REGISTRY_ID, SLOT_SIZE, MAX_ITEMS, RK, Format>
{
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn register<Spec>(
        &mut self,
        ident: MachineIdentificationUnique,
    ) -> Result<PropertyHandle<<Spec::Type as PropertyType>::Value>, RegisterError>
    where
        Spec: Specification + 'static,
        Spec::Type: PropertyType + Convertible<Format>,
        <Spec::Type as PropertyType>::Value: 'static,
    {
        const {
            assert!(size_of::<<Spec::Type as PropertyType>::Value>() <= 16);

            assert!(
                size_of::<<Spec::Type as PropertyType>::Value>() <= size_of::<Storage<SLOT_SIZE>>()
            );

            assert!(
                align_of::<<Spec::Type as PropertyType>::Value>()
                    <= align_of::<Storage<SLOT_SIZE>>()
            );
        }

        let index = self.find_free_slot(Spec::NAME)?;

        self.occupied[index] = true;
        self.buf_storage[index].write(Storage {
            bytes: [0u8; SLOT_SIZE],
        });

        let key = Key {
            ident,
            name: Spec::NAME,
        };
        let entry = Entry {
            index,
            converter: <Spec::Type as Convertible<Format>>::convert,
            spec_type_id: TypeId::of::<Spec>(),
        };

        self.lookup.insert(key, entry);

        let p_value = unsafe {
            NonNull::new_unchecked(
                self.buf_storage[index]
                    .as_mut_ptr()
                    .cast::<<Spec::Type as PropertyType>::Value>(),
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
                resource_kind: RK::KIND,
                resource_name,
                reason: RegisterErrorReason::RegistryFull,
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    ident: MachineIdentificationUnique,
    name: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct Entry<ExportFormat> {
    index: usize,
    spec_type_id: TypeId,
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
    const REGISTRY_ID: usize,
    const SLOT_SIZE: usize,
    const MAX_ITEMS: usize,
    RK: Kind,
    Format,
> {
    registry: &'a PropertyRegistry<REGISTRY_ID, SLOT_SIZE, MAX_ITEMS, RK, Format>,
}

impl<'a, const REGISTRY_ID: usize, const SLOT_SIZE: usize, const MAX_ITEMS: usize, RK: Kind, Format>
    PropertyReader<'a, REGISTRY_ID, SLOT_SIZE, MAX_ITEMS, RK, Format>
{
    pub fn new(
        registry: &'a PropertyRegistry<REGISTRY_ID, SLOT_SIZE, MAX_ITEMS, RK, Format>,
    ) -> Self {
        Self { registry }
    }

    pub fn read<Spec>(
        &self,
        handle: &PropertyAccessHandle<REGISTRY_ID, <Spec::Type as PropertyType>::Value>,
    ) -> Result<&<Spec::Type as PropertyType>::Value, ReadError>
    where
        Spec: Specification,
        Spec::Type: PropertyType,
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
            Ok(&*(storage.bytes.as_ptr() as *const <Spec::Type as PropertyType>::Value))
        }
    }
}

pub struct PropertyAccessHandle<const REGISTRY_ID: usize, T> {
    generation: u64,
    index: usize,
    _marker: PhantomData<T>,
}

pub struct PropertyResolver<
    'a,
    const REGISTRY_ID: usize,
    const SLOT_SIZE: usize,
    const MAX_ITEMS: usize,
    RK: Kind,
    Format,
> {
    registry: &'a PropertyRegistry<REGISTRY_ID, SLOT_SIZE, MAX_ITEMS, RK, Format>,
    ident: MachineIdentificationUnique,
}

impl<'a, const REGISTRY_ID: usize, const SLOT_SIZE: usize, const MAX_ITEMS: usize, RK: Kind, Format>
    PropertyResolver<'a, REGISTRY_ID, SLOT_SIZE, MAX_ITEMS, RK, Format>
{
    pub fn resolve<Spec>(
        &self,
    ) -> Result<PropertyAccessHandle<REGISTRY_ID, <Spec::Type as PropertyType>::Value>, ResolveError>
    where
        Spec: Specification + 'static,
        Spec::Type: PropertyType + Convertible<Format>,
        <Spec::Type as PropertyType>::Value: 'static,
    {
        let key = Key {
            ident: self.ident,
            name: Spec::NAME,
        };

        let Some(Entry {
            index,
            spec_type_id,
            ..
        }) = self.registry.lookup.get(&key)
        else {
            return Err(ResolveError::NoSuchProperty);
        };

        if *spec_type_id != TypeId::of::<Spec>() {
            return Err(ResolveError::NoSuchProperty);
        }

        let generation = unsafe { self.registry.buf_generations[*index].assume_init() };

        Ok(PropertyAccessHandle {
            index: *index,
            generation,
            _marker: PhantomData,
        })
    }
}
