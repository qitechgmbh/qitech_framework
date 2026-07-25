use std::any::TypeId;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ptr::NonNull;

use qitech_framework_common::MachineIdentificationUnique;

use super::PropertyKind;
use super::error::RegisterError;
use super::error::RegisterErrorKind;
use super::error::RegisterResult;
use super::error::ResolveError;
use super::error::ResolveErrorKind;
use super::error::ResolveResult;

pub struct PropertyManager<
    const SLOT_SIZE: usize,
    const MAX_ITEMS: usize,
    K: PropertyKind,
    Metadata = (),
> {
    occupied: heapless::Vec<bool, MAX_ITEMS>,
    buf_generation: [MaybeUninit<u64>; MAX_ITEMS],
    buf_storage: [MaybeUninit<Storage<SLOT_SIZE>>; MAX_ITEMS],
    buf_info: [MaybeUninit<SlotInfo<Metadata>>; MAX_ITEMS],
    _marker: PhantomData<K>,
}

impl<const SLOT_SIZE: usize, const MAX_ITEMS: usize, K, M> Default
    for PropertyManager<SLOT_SIZE, MAX_ITEMS, K, M>
where
    K: PropertyKind,
{
    fn default() -> Self {
        Self {
            occupied: Default::default(),
            buf_generation: [MaybeUninit::uninit(); MAX_ITEMS],
            buf_storage: [MaybeUninit::uninit(); MAX_ITEMS],
            buf_info: [const { MaybeUninit::uninit() }; MAX_ITEMS],
            _marker: PhantomData,
        }
    }
}

impl<const SLOT_SIZE: usize, const MAX_ITEMS: usize, K, M>
    PropertyManager<SLOT_SIZE, MAX_ITEMS, K, M>
where
    K: PropertyKind,
{
    pub fn register<T: 'static>(
        &mut self,
        ident: MachineIdentificationUnique,
        path: &'static str,
        path_postfix: &'static str,
        metadata: M,
        initial_value: T,
    ) -> RegisterResult<PropertyHandle<T>> {
        const {
            assert!(size_of::<T>() <= size_of::<Storage<SLOT_SIZE>>());
            assert!(align_of::<T>() <= align_of::<Storage<SLOT_SIZE>>());
        }

        let index = self.find_slot(ident, path, path_postfix)?;

        self.occupied[index] = true;

        let type_id = TypeId::of::<T>();

        self.buf_info[index].write(SlotInfo {
            ident,
            path,
            path_postfix,
            type_id,
            metadata,
        });

        let p_value = unsafe {
            let ptr = self.buf_storage[index].as_mut_ptr().cast::<T>();

            // initialize value
            *ptr = initial_value;

            NonNull::new_unchecked(ptr)
        };

        Ok(PropertyHandle { p_value })
    }

    pub fn unregister_machine(&mut self, ident: MachineIdentificationUnique) {
        for (i, occupied) in self.occupied.iter_mut().enumerate() {
            let info = unsafe { self.buf_info[i].assume_init_ref() };

            if info.ident != ident {
                continue;
            }

            // increment generation so existing handles to this resource fail
            let generation = unsafe { self.buf_generation[i].assume_init_mut() };
            *generation += 1;

            // mark slot as unused
            *occupied = false;
        }
    }

    pub fn resolve_read_handle<T: 'static>(
        &mut self,
        ident: MachineIdentificationUnique,
        path: &'static str,
    ) -> ResolveResult<PropertyReadHandle<K, T>> {
        let result = self.occupied.iter().enumerate().find_map(|(i, occupied)| {
            if !*occupied {
                return None;
            }

            let info = unsafe { self.buf_info[i].assume_init_ref() };

            (info.ident == ident && info.path == path).then_some((i, info))
        });

        let Some((index, info)) = result else {
            return Err(ResolveError {
                resource_kind: K::RESOURCE_KIND,
                resource_path: path,
                error_kind: ResolveErrorKind::NoSuchProperty,
            });
        };

        if info.type_id != TypeId::of::<T>() {
            return Err(ResolveError {
                resource_kind: K::RESOURCE_KIND,
                resource_path: path,
                error_kind: ResolveErrorKind::InvalidType,
            });
        }

        let generation = unsafe { self.buf_generation[index].assume_init() };

        Ok(PropertyReadHandle {
            index,
            generation,
            _marker: PhantomData,
        })
    }

    /// attempts to read data of an entry using a read handle
    pub fn read_value<T>(&self, handle: &PropertyReadHandle<K, T>) -> &T {
        let generation = &self.buf_generation[handle.index];

        // Safety:
        // - register() verified T fits in storage
        // - register() stored TypeId::of::<T>()
        // - resolve_read_handle() only creates PropertyHandle<T> after type check
        unsafe {
            // if generations don't match a machine attempted to read data with handles
            // after the subscription was terminated. This considered illegal
            assert_eq!(generation.assume_init(), handle.generation);

            let storage = &self.buf_storage[handle.index].assume_init_read();
            &*(storage.bytes.as_ptr() as *const T)
        }
    }

    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, SLOT_SIZE, MAX_ITEMS, K, M> {
        IterMut {
            manager: self,
            index: 0,
        }
    }

    // --- utils ---
    fn find_slot(
        &mut self, 
        ident: MachineIdentificationUnique, 
        path: &'static str,
        post: &'static str,
    ) -> RegisterResult<usize> {
        // --- step one: ensure no duplicates ---
        for item in &self.occupied {

        }

        if let Some(index) = self.occupied.iter().position(|slot| !slot) {
            return Ok(index);
        }

        if self.occupied.push(true).is_err() {
            return Err(RegisterError {
                resource_kind: K::RESOURCE_KIND,
                resource_path: path,
                error_kind: RegisterErrorKind::RegistryFull,
            });
        }

        Ok(self.occupied.len() - 1)
    }
}

// insert, write/read
pub struct SlotInfo<Metadata> {
    pub ident: MachineIdentificationUnique,
    pub path: &'static str,
    pub path_postfix: &'static str,
    pub type_id: TypeId,
    pub metadata: Metadata,
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
#[repr(C, align(16))]
struct Storage<const SLOT_SIZE: usize> {
    bytes: [u8; SLOT_SIZE],
}

pub struct PropertyReadHandle<K: PropertyKind, T> {
    generation: u64,
    index: usize,
    _marker: PhantomData<(K, T)>,
}

// --- iter ---
pub struct IterMut<'a, const SLOT_SIZE: usize, const MAX_ITEMS: usize, K, M>
where
    K: PropertyKind,
{
    manager: &'a mut PropertyManager<SLOT_SIZE, MAX_ITEMS, K, M>,
    index: usize,
}

impl<'a, const SLOT_SIZE: usize, const MAX_ITEMS: usize, K, M> Iterator
    for IterMut<'a, SLOT_SIZE, MAX_ITEMS, K, M>
where
    K: PropertyKind,
{
    type Item = (SlotInfo<M>, *const u8);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.manager.occupied.len() {
            let index = self.index;
            self.index += 1;

            if self.manager.occupied[index] {
                todo!("yield occupied slot {index}");
            }
        }

        None
    }
}
