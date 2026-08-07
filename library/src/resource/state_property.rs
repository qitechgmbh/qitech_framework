use std::any::TypeId;
use std::any::type_name;
use std::mem::MaybeUninit;
use std::ptr::NonNull;

use chrono::Utc;
use qitech_framework_core::ScalarValue;
use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::report::ResourceAccessError;
use qitech_framework_core::report::ResourceKind;
use qitech_framework_core::report::StatePropertyEvent;
use qitech_framework_core::report::StatePropertyRecord;

use crate::resource::BumpAllocator;
use crate::resource::BumpAllocatorMark;
use crate::resource::CachedPropertyView;
use crate::resource::JournalHandle;
use crate::resource::MachineInfo;
use crate::resource::SlotInfo;
use crate::resource::SlotState;
use crate::resource::conversion::PropertyType;

pub struct StateProperty<T: PropertyType> {
    handle: StatePropertyHandle<T>,

    // --- conversion functions ---
    into_scalar: fn(T) -> ScalarValue,

    // --- journals ---
    journal_value: JournalHandle<StatePropertyRecord>,
}

impl<T: PropertyType> StateProperty<T> {
    pub(crate) fn new(
        handle: StatePropertyHandle<T>,
        into_scalar: fn(T) -> ScalarValue,
        journal_value: JournalHandle<StatePropertyRecord>,
    ) -> Self {
        Self {
            handle,
            into_scalar,
            journal_value,
        }
    }

    pub fn get_ref(&self) -> &T {
        self.validate();
        unsafe { self.handle.p_value.as_ref() }
    }

    pub fn set(&mut self, value: T) {
        self.validate();

        let current = unsafe { self.handle.p_value.as_ref() };

        // state property doesn't record if value is unchanged
        if *current == value {
            return;
        }

        // --- write the value ---
        unsafe {
            self.handle.p_value.write(value.clone());
        }

        // --- record the change ---
        let value = (self.into_scalar)(value);
        let descriptor = unsafe { self.handle.p_desc.read() };

        self.journal_value.append(StatePropertyRecord {
            timestamp: Utc::now(),
            machine: descriptor.ident,
            path: descriptor.path.to_string(),
            event: StatePropertyEvent::ValueChanged { value },
        });
    }

    /// ensures that the slot is activated and the generation matches
    fn validate(&self) {
        let slot_info = unsafe { self.handle.p_info.read() };
        assert_eq!(slot_info.state, SlotState::Activated);
        assert_eq!(slot_info.generation, self.handle.generation)
    }
}

pub struct StatePropertyHandle<T: PropertyType> {
    generation: u64,
    p_info: NonNull<SlotInfo>,
    p_desc: NonNull<SlotDescriptor>,
    p_value: NonNull<T>,
}

// --- registry ---
pub struct StatePropertyRegistry {
    machines: heapless::Vec<MachineInfo, 64>,

    // --- item ---
    pool_items_pos: usize,
    pool_slot: Box<[SlotInfo]>,
    pool_desc: Box<[MaybeUninit<SlotDescriptor>]>,

    // --- value ---
    alloc_value: BumpAllocator,
    alloc_cache: BumpAllocator,
}

impl StatePropertyRegistry {
    pub fn new(pool_size: usize, items_max: usize) -> Self {
        Self {
            machines: heapless::Vec::new(),

            pool_items_pos: 0,
            pool_slot: vec![SlotInfo::default(); items_max].into_boxed_slice(),
            pool_desc: vec![MaybeUninit::uninit(); items_max].into_boxed_slice(),

            alloc_value: BumpAllocator::new(pool_size),
            alloc_cache: BumpAllocator::new(pool_size),
        }
    }

    pub fn deactivate_machine(&mut self, ident: MachineIdentificationUnique) {
        let machine = self
            .machines
            .iter()
            .find(|m| m.ident == ident)
            .expect("machine not registered");

        // invalidate all current properties. Using any property
        // associated with this machine will lead to a panic.
        for index in machine.pos..machine.pos + machine.len {
            self.pool_slot[index].state = SlotState::Deactivated;
            self.pool_slot[index].generation += 1;
        }
    }

    pub fn register_machine(
        &'_ mut self,
        ident: MachineIdentificationUnique,
    ) -> StatePropertyRegistryRegisterHandle<'_> {
        let item_pos = self.pool_items_pos;
        let value_mark = self.alloc_value.mark();
        let cache_mark = self.alloc_cache.mark();

        StatePropertyRegistryRegisterHandle {
            registry: self,
            ident,
            item_pos,
            value_mark,
            cache_mark,
            committed: false,
        }
    }

    pub fn sync_cache(&mut self) {
        // --- copy snapshot of values into cache ---
        self.alloc_cache.sync(&self.alloc_value);
    }

    // TODO: return result
    pub fn new_cached_view<T: Clone + 'static>(
        &self,
        ident: MachineIdentificationUnique,
        resource: &str,
    ) -> Result<CachedPropertyView<T>, ResourceAccessError> {
        let Some(entry) = self.machines.iter().find(|m| m.ident == ident) else {
            return Err(ResourceAccessError::MachineNotFound(ident));
        };

        for i in entry.pos..entry.pos + entry.len {
            if self.pool_slot[i].state != SlotState::Activated {
                continue;
            }

            let descriptor = unsafe { self.pool_desc[i].assume_init_ref() };

            if descriptor.path != resource {
                continue;
            }

            if descriptor.type_id != TypeId::of::<T>() {
                return Err(ResourceAccessError::TypeMismatch {
                    actual: descriptor.type_name.to_string(),
                    received: type_name::<T>().to_string(),
                });
            }

            let p_value = unsafe { NonNull::new_unchecked(descriptor.p_cache as *mut T) };

            return Ok(CachedPropertyView::new(p_value));
        }

        Err(ResourceAccessError::ResourceNotFound {
            path: resource.to_string(),
            kind: ResourceKind::StateProperty,
        })
    }
}

#[derive(Clone, Copy)]
pub struct SlotDescriptor {
    type_id: TypeId,
    type_name: &'static str,
    ident: MachineIdentificationUnique,
    path: &'static str,
    p_cache: *mut (),
}

pub struct StatePropertyRegistryRegisterHandle<'a> {
    registry: &'a mut StatePropertyRegistry,
    ident: MachineIdentificationUnique,
    item_pos: usize,
    value_mark: BumpAllocatorMark,
    cache_mark: BumpAllocatorMark,
    committed: bool,
}

impl<'a> StatePropertyRegistryRegisterHandle<'a> {
    pub fn register<T: PropertyType>(
        &mut self,
        path: &'static str,
        value: T,
    ) -> StatePropertyHandle<T> {
        let index = self.registry.pool_items_pos;

        assert!(
            index < self.registry.pool_desc.len(),
            "Property registry exhausted"
        );

        let p_value = self.registry.alloc_value.allocate::<T>();
        let p_cache = self.registry.alloc_cache.allocate::<T>();

        assert_eq!(
            self.registry.alloc_value.used(),
            self.registry.alloc_cache.used(),
            "value/cache allocator layout diverged"
        );

        unsafe {
            p_value.write(value.clone());
            p_cache.write(value.clone());
        }

        let descriptor = SlotDescriptor {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            ident: self.ident,
            path,
            p_cache: p_cache.as_ptr() as *mut (),
        };

        self.registry.pool_desc[index].write(descriptor);
        self.registry.pool_items_pos += 1;

        let p_desc = unsafe {
            NonNull::new_unchecked(
                self.registry
                    .pool_desc
                    .as_mut_ptr()
                    .add(index)
                    .cast::<SlotDescriptor>(),
            )
        };

        let p_info =
            unsafe { NonNull::new_unchecked(&mut self.registry.pool_slot[index] as *mut SlotInfo) };

        StatePropertyHandle {
            generation: self.registry.pool_slot[index].generation,
            p_info,
            p_desc,
            p_value,
        }
    }

    pub fn commit(mut self) {
        for index in self.item_pos..self.registry.pool_items_pos {
            let slot_info = &mut self.registry.pool_slot[index];
            slot_info.state = SlotState::Activated;
        }

        self.registry
            .machines
            .push(MachineInfo {
                ident: self.ident,
                pos: self.item_pos,
                len: self.registry.pool_items_pos - self.item_pos,
            })
            .expect("machine registry exhausted");

        self.committed = true;
    }
}

impl Drop for StatePropertyRegistryRegisterHandle<'_> {
    fn drop(&mut self) {
        if self.committed {
            return;
        }

        // mark all reserved slots as unused again and increment generation
        // so any created handles cannot access the memory
        for index in self.item_pos..self.registry.pool_items_pos {
            let slot_info = &mut self.registry.pool_slot[index];
            slot_info.state = SlotState::Unused;
            slot_info.generation += 1;
        }

        // --- roll back item count ---
        self.registry.pool_items_pos = self.item_pos;

        // Revert the bump allocations
        self.registry.alloc_value.rollback(self.value_mark);
        self.registry.alloc_cache.rollback(self.cache_mark);
    }
}
