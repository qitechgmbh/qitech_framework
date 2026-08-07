use std::any::TypeId;
use std::any::type_name;
use std::mem;
use std::mem::MaybeUninit;
use std::ptr::NonNull;

use chrono::Utc;
use qitech_framework_core::ScalarValue;
use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::report::ConfigPropertyEvent;
use qitech_framework_core::report::ConfigPropertyRecord;
use qitech_framework_core::report::ConfigPropertyWriteError;
use qitech_framework_core::report::ConfigPropertyWriteOutcome;
use qitech_framework_core::report::ConstraintViolationError;
use qitech_framework_core::report::Constraints;
use qitech_framework_core::report::OperationOrigin;
use qitech_framework_core::report::ResourceAccessError;
use qitech_framework_core::report::ResourceKind;
use qitech_framework_core::report::WriteCapability;
use qitech_framework_core::with_uom_quantities;

use super::Machine;
use crate::resource::BumpAllocator;
use crate::resource::CachedPropertyView;
use crate::resource::Erased;
use crate::resource::JournalHandle;
use crate::resource::MachineInfo;
use crate::resource::OnExternalChangedCallback;
use crate::resource::PropertyDescriptor;
use crate::resource::SlotInfo;
use crate::resource::SlotState;
use crate::resource::bump_allocator::BumpAllocatorMark;
use crate::resource::conversion::PropertyType;

pub struct ConfigProperty<T: PropertyType> {
    handle: ConfigPropertyHandle<T>,

    // --- conversion functions ---
    into_scalar: fn(T) -> ScalarValue,
    validate_constraints: fn(&T::Constraints, &T) -> Result<(), ConstraintViolationError>,
    as_parameter_constraints: fn(&T::Constraints) -> Constraints,

    // --- journals ---
    journal: JournalHandle<ConfigPropertyRecord>,
}

impl<T: PropertyType> ConfigProperty<T> {
    pub(crate) fn new(
        handle: ConfigPropertyHandle<T>,
        into_scalar: fn(T) -> ScalarValue,
        validate_constraints: fn(&T::Constraints, &T) -> Result<(), ConstraintViolationError>,
        as_parameter_constraints: fn(&T::Constraints) -> Constraints,
        journal: JournalHandle<ConfigPropertyRecord>,
    ) -> Self {
        Self {
            handle,
            into_scalar,
            validate_constraints,
            as_parameter_constraints,
            journal,
        }
    }

    pub fn get_ref(&self) -> &T {
        self.validate();
        unsafe { self.handle.p_value.as_ref() }
    }

    pub fn set(&mut self, value: T) -> Result<bool, ConstraintViolationError> {
        self.validate();

        if value == *self.get_ref() {
            self.record(ConfigPropertyEvent::Written {
                value: (self.into_scalar)(value),
                origin: OperationOrigin::Machine,
                outcome: ConfigPropertyWriteOutcome::Accepted { changed: false },
            });

            return Ok(false);
        }

        let input = (self.into_scalar)(value.clone());
        let res = self.write(value.clone());

        match &res {
            Ok(_) => {
                self.record(ConfigPropertyEvent::Written {
                    value: input,
                    origin: OperationOrigin::Machine,
                    outcome: ConfigPropertyWriteOutcome::Accepted { changed: true },
                });
            }

            Err(e) => {
                let err = ConfigPropertyWriteError::ConstraintViolation(e.clone());
                self.record(ConfigPropertyEvent::Written {
                    value: input,
                    origin: OperationOrigin::Machine,
                    outcome: ConfigPropertyWriteOutcome::Rejected(err),
                });
            }
        }

        res.map(|_| true)
    }

    /// resets property back to the assigned default value
    pub fn reset(&mut self) -> Result<bool, ConstraintViolationError> {
        self.validate();

        unsafe {
            let default = &self.handle.p_state.as_ref().default_value;
            self.set(default.clone())
        }
    }

    pub fn allow_external_write(&mut self) {
        self.set_writable(WriteCapability::Allowed);
    }

    pub fn forbid_external_write(&mut self, reason: impl Into<String>) {
        self.set_writable(WriteCapability::Forbidden {
            reason: reason.into(),
        });
    }

    fn set_writable(&mut self, value: WriteCapability) {
        self.validate();

        let state = unsafe { self.handle.p_state.as_mut() };

        if value == state.writable {
            return;
        }

        let before = mem::replace(&mut state.writable, value);
        self.record(ConfigPropertyEvent::CapabilityChanged {
            before,
            after: state.writable.clone(),
        });
    }

    pub fn set_default(&mut self, value: T) -> Result<bool, ConstraintViolationError> {
        self.validate();

        let state = unsafe { self.handle.p_state.as_mut() };

        if value == state.default_value {
            return Ok(false);
        }

        // ensure new default value adheres to the constraints
        (self.validate_constraints)(&state.constraints, &value)?;

        let before = mem::replace(&mut state.default_value, value);
        self.record(ConfigPropertyEvent::DefaultChanged {
            before: (self.into_scalar)(before),
            after: (self.into_scalar)(state.default_value.clone()),
        });

        Ok(true)
    }

    pub fn constraints(&self) -> &T::Constraints {
        self.validate();
        let state = unsafe { self.handle.p_state.as_ref() };
        &state.constraints
    }

    // TODO: don't let strings mutate constraints !!!
    pub fn set_constraints(
        &mut self,
        constraints: T::Constraints,
    ) -> Result<bool, ConstraintViolationError> {
        self.validate();
        let state = unsafe { self.handle.p_state.as_mut() };

        if constraints == state.constraints {
            return Ok(false);
        }

        // ensure both current and default value are still valid with new constraints
        (self.validate_constraints)(&constraints, self.get_ref())?;
        (self.validate_constraints)(&constraints, &state.default_value)?;

        let before = (self.as_parameter_constraints)(&state.constraints);

        state.constraints = constraints;

        let after = (self.as_parameter_constraints)(&state.constraints);

        self.record(ConfigPropertyEvent::ConstraintsChanged { before, after });
        Ok(true)
    }

    // --- utils ---
    fn descriptor(&self) -> PropertyDescriptor {
        unsafe { self.handle.p_descriptor.read() }
    }

    fn write(&mut self, value: T) -> Result<(), ConstraintViolationError> {
        unsafe {
            let state = self.handle.p_state.as_ref();
            (self.validate_constraints)(&state.constraints, &value)?;
            self.handle.p_value.write(value);
        }

        Ok(())
    }

    /// ensures that the slot is activated and the generation matches
    fn validate(&self) {
        let slot_info = unsafe { self.handle.p_slot.read() };
        assert_eq!(slot_info.state, SlotState::Activated);
        assert_eq!(slot_info.generation, self.handle.generation)
    }

    fn record(&mut self, event: ConfigPropertyEvent) {
        let descriptor = self.descriptor();
        self.journal.append(ConfigPropertyRecord {
            timestamp: Utc::now(),
            machine: descriptor.ident,
            path: descriptor.path.to_string(),
            event,
        });
    }
}

impl<T: PropertyType + Copy> ConfigProperty<T> {
    pub fn get(&self) -> T {
        self.validate();
        unsafe { self.handle.p_value.read() }
    }
}

// --- uom impl ---
macro_rules! impl_uom {
    ($quantity:path, $unit_trait:path, $conversion_trait:path) => {
        impl ConfigProperty<$quantity> {
            pub fn get_as<N>(&self) -> f64
            where
                N: $unit_trait + $conversion_trait,
            {
                self.get().get::<N>()
            }

            pub fn set_as<N>(&mut self, value: f64) -> Result<bool, ConstraintViolationError>
            where
                N: $unit_trait + $conversion_trait,
            {
                self.set(<$quantity>::new::<N>(value))
            }
        }

        impl ConfigProperty<Option<$quantity>> {
            pub fn get_as<N>(&self) -> Option<f64>
            where
                N: $unit_trait + $conversion_trait,
            {
                self.get().map(|q| q.get::<N>())
            }

            pub fn set_as<N>(
                &mut self,
                value: Option<f64>,
            ) -> Result<bool, ConstraintViolationError>
            where
                N: $unit_trait + $conversion_trait,
            {
                self.set(value.map(<$quantity>::new::<N>))
            }
        }
    };
}

with_uom_quantities!(impl_uom);

pub struct ConfigPropertyHandle<T: PropertyType> {
    generation: u64,
    p_slot: NonNull<SlotInfo>,
    p_descriptor: NonNull<PropertyDescriptor>,
    p_value: NonNull<T>,
    p_state: NonNull<ConfigPropertyState<T>>,
}

#[derive(Clone, Copy)]
pub struct ExecuteContext {
    /// type-erased property value
    value: Erased,

    /// type-erased property state
    state: Erased,

    /// function to write scalar value into the property.
    /// returns Some if value changed, None otherwise
    write: fn(Erased, ScalarValue, Erased) -> Result<bool, ConfigPropertyWriteError>,

    /// callback invoked when a external write succeeds and the value changed
    on_external_changed: Option<OnExternalChangedCallback>,
}

impl ExecuteContext {
    pub fn execute(
        self,
        machine: &mut dyn Machine,
        value: ScalarValue,
    ) -> Result<bool, ConfigPropertyWriteError> {
        let result = (self.write)(self.state, value, self.value);

        if let Some(callback) = self.on_external_changed
            && result.is_ok()
        {
            (callback.adapter)(machine, callback.func);
        }

        result
    }
}

pub struct ConfigPropertyState<T: PropertyType> {
    pub(crate) default_value: T,
    pub(crate) constraints: T::Constraints,
    pub(crate) writable: WriteCapability,
}

pub struct ConfigPropertyRegistry {
    machines: heapless::Vec<MachineInfo, 64>,

    buf_len: usize,
    buf_slot_info: Box<[SlotInfo]>,
    buf_descriptor: Box<[MaybeUninit<PropertyDescriptor>]>,
    pool_exec_ctx: Box<[MaybeUninit<ExecuteContext>]>,

    alloc_value: BumpAllocator,
    alloc_cache: BumpAllocator,
    alloc_state: BumpAllocator,
}

impl ConfigPropertyRegistry {
    pub fn new(pool_size: usize, items_max: usize) -> Self {
        Self {
            machines: heapless::Vec::new(),

            buf_len: 0,
            buf_slot_info: vec![SlotInfo::default(); items_max].into_boxed_slice(),
            buf_descriptor: vec![MaybeUninit::uninit(); items_max].into_boxed_slice(),
            pool_exec_ctx: vec![MaybeUninit::uninit(); items_max].into_boxed_slice(),

            alloc_value: BumpAllocator::new(pool_size),
            alloc_cache: BumpAllocator::new(pool_size),
            alloc_state: BumpAllocator::new(pool_size),
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
            self.buf_slot_info[index].state = SlotState::Deactivated;
            self.buf_slot_info[index].generation += 1;
        }
    }

    pub fn register_machine(
        &'_ mut self,
        ident: MachineIdentificationUnique,
    ) -> ConfigPropertyRegistryRegisterHandle<'_> {
        let item_pos = self.buf_len;
        let value_mark = self.alloc_value.mark();
        let cache_mark = self.alloc_cache.mark();
        let state_mark = self.alloc_state.mark();

        ConfigPropertyRegistryRegisterHandle {
            registry: self,
            ident,
            item_pos,
            value_mark,
            cache_mark,
            state_mark,
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
            if self.buf_slot_info[i].state != SlotState::Activated {
                continue;
            }

            let descriptor = unsafe { self.buf_descriptor[i].assume_init_ref() };

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
            kind: ResourceKind::ConfigProperty,
        })
    }

    pub fn execute_context(
        &self,
        ident: MachineIdentificationUnique,
        resource: &str,
    ) -> Option<ExecuteContext> {
        let machine_info = self.machines.iter().find(|m| m.ident == ident)?;

        let pos = machine_info.pos;
        let len = machine_info.len;

        for i in pos..pos + len {
            unsafe {
                let info = self.buf_descriptor[i].assume_init_read();

                if info.path == resource {
                    return Some(self.pool_exec_ctx[i].assume_init_read());
                }
            }
        }

        None
    }
}

/// Handle used to register properties for a given machine ident.
/// If .commit() is not used the bump allocator will be reset
/// and all allocated resources will be invalidated.
pub struct ConfigPropertyRegistryRegisterHandle<'a> {
    registry: &'a mut ConfigPropertyRegistry,
    ident: MachineIdentificationUnique,
    item_pos: usize,
    value_mark: BumpAllocatorMark,
    cache_mark: BumpAllocatorMark,
    state_mark: BumpAllocatorMark,
    committed: bool,
}

impl<'a> ConfigPropertyRegistryRegisterHandle<'a> {
    pub fn register<T: PropertyType>(
        &mut self,
        path: &'static str,
        default: T,
        writable: WriteCapability,
        constraints: T::Constraints,
        write: fn(Erased, ScalarValue, Erased) -> Result<bool, ConfigPropertyWriteError>,
        on_changed: Option<OnExternalChangedCallback>,
    ) -> ConfigPropertyHandle<T> {
        let index = self.registry.buf_len;

        assert!(
            index < self.registry.buf_descriptor.len(),
            "Property registry exhausted"
        );

        let p_value = self.registry.alloc_value.allocate::<T>();
        let p_cache = self.registry.alloc_cache.allocate::<T>();
        let p_state = self
            .registry
            .alloc_state
            .allocate::<ConfigPropertyState<T>>();

        assert_eq!(
            self.registry.alloc_value.used(),
            self.registry.alloc_cache.used(),
            "value/cache allocator layout diverged"
        );

        unsafe {
            p_value.write(default.clone());
            p_cache.write(default.clone());
            p_state.write(ConfigPropertyState {
                default_value: default,
                constraints,
                writable,
            });
        }

        let descriptor = PropertyDescriptor {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            ident: self.ident,
            path,
            p_value: p_value.as_ptr() as *mut (),
            p_cache: p_cache.as_ptr() as *mut (),
            p_state: p_state.as_ptr() as *mut (),
        };

        let exec_ctx = ExecuteContext {
            value: Erased::new(p_value),
            state: Erased::new(p_state),
            write,
            on_external_changed: on_changed,
        };

        self.registry.buf_descriptor[index].write(descriptor);
        self.registry.pool_exec_ctx[index].write(exec_ctx);
        self.registry.buf_len += 1;

        let p_descriptor = unsafe {
            NonNull::new_unchecked(
                self.registry
                    .buf_descriptor
                    .as_mut_ptr()
                    .add(index)
                    .cast::<PropertyDescriptor>(),
            )
        };

        let p_slot = unsafe {
            NonNull::new_unchecked(&mut self.registry.buf_slot_info[index] as *mut SlotInfo)
        };

        ConfigPropertyHandle {
            generation: self.registry.buf_slot_info[index].generation,
            p_slot,
            p_descriptor,
            p_state,
            p_value,
        }
    }

    pub fn commit(mut self) {
        for index in self.item_pos..self.registry.buf_len {
            let slot_info = &mut self.registry.buf_slot_info[index];
            slot_info.state = SlotState::Activated;
        }

        self.registry
            .machines
            .push(MachineInfo {
                ident: self.ident,
                pos: self.item_pos,
                len: self.registry.buf_len - self.item_pos,
            })
            .expect("machine registry exhausted");

        self.committed = true;
    }
}

impl Drop for ConfigPropertyRegistryRegisterHandle<'_> {
    fn drop(&mut self) {
        if self.committed {
            return;
        }

        // mark all reserved slots as unused again and increment generation
        // so any created handles cannot access the memory
        for index in self.item_pos..self.registry.buf_len {
            let slot_info = &mut self.registry.buf_slot_info[index];
            slot_info.state = SlotState::Unused;
            slot_info.generation += 1;
        }

        // --- roll back item count ---
        self.registry.buf_len = self.item_pos;

        // Revert the bump allocations
        self.registry.alloc_value.rollback(self.value_mark);
        self.registry.alloc_cache.rollback(self.cache_mark);
        self.registry.alloc_state.rollback(self.state_mark);
    }
}
