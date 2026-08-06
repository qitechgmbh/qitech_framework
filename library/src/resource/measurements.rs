use std::any::TypeId;
use std::mem::MaybeUninit;
use std::ptr::NonNull;

use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::with_uom_quantities;

use crate::resource::BumpAllocator;
use crate::resource::BumpAllocatorMark;
use crate::resource::MachineInfo;
use crate::resource::SlotInfo;
use crate::resource::SlotState;
use crate::resource::conversion::PropertyType;

pub struct Measurement<T: PropertyType> {
    handle: MeasurementHandle<T>,
}

impl<T: PropertyType + Copy> Measurement<T> {
    pub(crate) fn new(handle: MeasurementHandle<T>) -> Self {
        Self { handle }
    }

    pub fn get(&self) -> T {
        self.validate();
        unsafe { self.handle.p_value.read() }
    }

    pub fn set(&mut self, value: T) {
        self.validate();

        unsafe {
            self.handle.p_value.write(value);
        }
    }

    /// ensures that the slot is activated and the generation matches
    fn validate(&self) {
        let slot_info = unsafe { self.handle.p_info.read() };
        assert_eq!(slot_info.state, SlotState::Activated);
        assert_eq!(slot_info.generation, self.handle.generation)
    }
}

// --- uom impl ---
macro_rules! impl_uom {
    ($quantity:path, $unit_trait:path, $conversion_trait:path) => {
        impl Measurement<$quantity> {
            pub fn get_as<N>(&self) -> f64
            where
                N: $unit_trait + $conversion_trait,
            {
                self.get().get::<N>()
            }

            pub fn set_as<N>(&mut self, value: f64)
            where
                N: $unit_trait + $conversion_trait,
            {
                self.set(<$quantity>::new::<N>(value))
            }
        }

        impl Measurement<Option<$quantity>> {
            pub fn get_as<N>(&self) -> Option<f64>
            where
                N: $unit_trait + $conversion_trait,
            {
                self.get().map(|q| q.get::<N>())
            }

            pub fn set_as<N>(&mut self, value: Option<f64>)
            where
                N: $unit_trait + $conversion_trait,
            {
                self.set(value.map(<$quantity>::new::<N>))
            }
        }
    };
}

with_uom_quantities!(impl_uom);

pub struct MeasurementHandle<T: PropertyType> {
    generation: u64,
    p_info: NonNull<SlotInfo>,
    p_desc: NonNull<SlotDescriptor>,
    p_value: NonNull<T>,
}

// --- registry ---
pub struct MeasurementRegistry {
    machines: heapless::Vec<MachineInfo, 64>,

    // --- item ---
    pool_items_pos: usize,
    pool_slot: Box<[SlotInfo]>,
    pool_desc: Box<[MaybeUninit<SlotDescriptor>]>,

    // --- value ---
    alloc_value: BumpAllocator,
    alloc_cache: BumpAllocator,
}

impl MeasurementRegistry {
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
    ) -> MeasurementRegistryRegisterHandle<'_> {
        let item_pos = self.pool_items_pos;
        let value_mark = self.alloc_value.mark();
        let cache_mark = self.alloc_cache.mark();

        MeasurementRegistryRegisterHandle {
            registry: self,
            ident,
            item_pos,
            value_mark,
            cache_mark,
            committed: false,
        }
    }

    pub fn sync(&mut self) {
        // --- copy snapshot of values into cache ---
        self.alloc_cache.sync(&self.alloc_value);
    }
}

#[derive(Clone, Copy)]
pub struct SlotDescriptor {
    // --- metadata ---
    ident: MachineIdentificationUnique,
    resource: &'static str,
    stat_name: Option<&'static str>,

    // --- typed info ---
    type_id: TypeId,
    p_value: *mut (),
    p_cache: *mut (),
}

pub struct MeasurementRegistryRegisterHandle<'a> {
    registry: &'a mut MeasurementRegistry,
    ident: MachineIdentificationUnique,
    item_pos: usize,
    value_mark: BumpAllocatorMark,
    cache_mark: BumpAllocatorMark,
    committed: bool,
}

impl<'a> MeasurementRegistryRegisterHandle<'a> {
    pub fn register<T: PropertyType>(
        &mut self,
        path: &'static str,
        value: T,
    ) -> MeasurementHandle<T> {
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
            ident: self.ident,
            resource: path,
            stat_name: None,
            p_value: p_value.as_ptr() as *mut (),
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

        MeasurementHandle {
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

impl Drop for MeasurementRegistryRegisterHandle<'_> {
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
