use std::any::TypeId;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ptr::NonNull;
use std::rc::Weak;

use qitech_framework_common::MachineIdentificationUnique;

use super::PropertyKind;
use super::error::RegisterErrorKind;
use super::error::RegisterResult;
use super::error::SubscribeErrorKind;
use super::error::SubscribeResult;
use crate::machine::resource::ResourceKey;
use crate::machine::resource::SubscriptionRegistry;
use crate::machine::resource::SubscriptionToken;

// create new property, stored under a ResourceId

pub struct PropertyManager<
    const SLOT_SIZE: usize,
    const MAX_ITEMS: usize,
    K: PropertyKind,
    Metadata = (),
> {
    occupied: heapless::Vec<bool, MAX_ITEMS>,
    buf_gen: [u64; MAX_ITEMS],
    buf_info: [MaybeUninit<SlotInfo<Metadata>>; MAX_ITEMS],
    buf_storage: [MaybeUninit<Storage<SLOT_SIZE>>; MAX_ITEMS],
    buf_cache: [MaybeUninit<Storage<SLOT_SIZE>>; MAX_ITEMS],
    subscriptions: SubscriptionRegistry,
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
            buf_gen: [0; MAX_ITEMS],
            buf_info: [const { MaybeUninit::uninit() }; MAX_ITEMS],
            buf_storage: [MaybeUninit::uninit(); MAX_ITEMS],
            buf_cache: [MaybeUninit::uninit(); MAX_ITEMS],
            subscriptions: Default::default(),
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
        path: String,
        metadata: M,
        initial_value: T,
    ) -> RegisterResult<PropertyHandle<T>> {
        const {
            assert!(size_of::<T>() <= size_of::<Storage<SLOT_SIZE>>());
            assert!(align_of::<T>() <= align_of::<Storage<SLOT_SIZE>>());
        }

        let index = self.find_slot()?;

        self.occupied[index] = true;
        let type_id = TypeId::of::<T>();

        self.buf_info[index].write(SlotInfo {
            machine: ident,
            path,
            type_id,
            metadata,
        });

        let generation = self.buf_gen[index];

        let p_generation = unsafe {
            let ptr = self.buf_gen[index] as *mut u64;
            NonNull::new_unchecked(ptr)
        };

        let p_value = unsafe {
            let ptr = self.buf_storage[index].as_mut_ptr().cast::<T>();

            // initialize value
            *ptr = initial_value;

            NonNull::new_unchecked(ptr)
        };

        Ok(PropertyHandle {
            generation,
            p_generation,
            p_value,
        })
    }

    pub fn unregister_machine(&mut self, ident: MachineIdentificationUnique) {
        for (i, occupied) in self.occupied.iter_mut().enumerate() {
            let info = unsafe { self.buf_info[i].assume_init_ref() };

            if info.machine != ident {
                continue;
            }

            // increment generation so existing handles to this resource fail
            self.buf_gen[i] += 1;

            // mark slot as unused
            *occupied = false;
        }

        self.subscriptions.unregister_producer(ident);
    }

    pub fn create_subscriber<T: 'static>(
        &mut self,
        provider: MachineIdentificationUnique,
        subscriber: MachineIdentificationUnique,
        resource: &'static str,
    ) -> SubscribeResult<Subscriber<T>> {
        let result = self.occupied.iter().enumerate().find_map(|(i, occupied)| {
            if !*occupied {
                return None;
            }

            let info = unsafe { self.buf_info[i].assume_init_ref() };
            (info.machine == provider && info.path == resource).then_some((i, info))
        });

        let Some((index, info)) = result else {
            return Err(SubscribeErrorKind::NoSuchProperty);
        };

        if info.type_id != TypeId::of::<T>() {
            return Err(SubscribeErrorKind::InvalidType);
        }

        let p_value = unsafe {
            let ptr = self.buf_cache[index].as_mut_ptr().cast::<T>();
            NonNull::new_unchecked(ptr)
        };

        let token = self.subscriptions.register(provider, subscriber, resource)?;
        Ok(Subscriber { token, p_value })
    }

    pub fn remove_subscription(
        &mut self, 
        provider: MachineIdentificationUnique,
        consumer: MachineIdentificationUnique,
    ) {
        self.subscriptions.unregister(provider, consumer);
    }

    pub fn sync_cache(&mut self) {
        for (i, occupied) in self.occupied.iter().enumerate() {
            if !occupied {
                continue;
            }
            self.buf_cache[i] = self.buf_storage[i];
        }
    }

    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, SLOT_SIZE, MAX_ITEMS, K, M> {
        IterMut {
            manager: self,
            index: 0,
        }
    }

    // --- utils ---
    fn find_slot(&mut self) -> RegisterResult<usize> {
        // --- step one: ensure no duplicates ---
        for item in &self.occupied {}

        if let Some(index) = self.occupied.iter().position(|slot| !slot) {
            return Ok(index);
        }

        if self.occupied.push(true).is_err() {
            return Err(RegisterErrorKind::RegistryFull);
        }

        Ok(self.occupied.len() - 1)
    }
}

// insert, write/read
pub struct SlotInfo<Metadata> {
    pub machine: MachineIdentificationUnique,
    pub path: String,
    pub type_id: TypeId,
    pub metadata: Metadata,
}

#[derive(Debug)]
pub struct PropertyHandle<T> {
    generation: u64,
    p_generation: NonNull<u64>,
    p_value: NonNull<T>,
}

impl<T> PropertyHandle<T> {
    pub fn read(&self) -> &T {
        unsafe {
            assert!(self.generation == self.p_generation.read());
            self.p_value.as_ref()
        }
    }

    pub fn write(&self, value: T) {
        unsafe {
            assert!(self.generation == self.p_generation.read());
            self.p_value.write(value)
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C, align(16))]
struct Storage<const SLOT_SIZE: usize> {
    bytes: [u8; SLOT_SIZE],
}

pub struct Subscriber<T> {
    token: Weak<SubscriptionToken>,
    p_value: NonNull<T>,
}

impl<T> Subscriber<T> {
    pub fn get_ref(&self) -> &T {
        self.token
            .upgrade()
            .expect("Subscriber outlived subscription");
        unsafe { self.p_value.as_ref() }
    }
}

impl<T: Copy> Subscriber<T> {
    pub fn get(&self) -> T {
        self.token
            .upgrade()
            .expect("Subscriber outlived subscription");
        unsafe { self.p_value.read() }
    }
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
