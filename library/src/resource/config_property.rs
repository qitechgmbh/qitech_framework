use std::any::TypeId;
use std::mem::MaybeUninit;
use std::ptr::NonNull;

use chrono::Utc;
use qitech_framework_core::ScalarValue;
use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::report::ConfigPropertyStateChange;
use qitech_framework_core::report::ConfigPropertyStateRecord;
use qitech_framework_core::report::ConfigPropertyValueRecord;
use qitech_framework_core::report::ConfigPropertyWriteError;
use qitech_framework_core::report::ConfigPropertyWriteResult;
use qitech_framework_core::report::ConstraintViolation;
use qitech_framework_core::report::OperationOrigin;
use qitech_framework_core::report::ParameterConstraints;
use qitech_framework_core::report::WriteCapability;

use super::Machine;
use crate::resource::BumpAllocator;
use crate::resource::JournalHandle;
use crate::resource::MachineInfo;
use crate::resource::OnExternalChangedCallback;
use crate::resource::PropertyDescriptor;
use crate::resource::SlotState;
use crate::resource::bump_allocator::BumpAllocatorMark;
use crate::resource::conversion::PropertyType;

pub struct ConfigProperty<T: PropertyType> {
    handle: ConfigPropertyHandle<T>,

    // --- conversion functions ---
    into_scalar: fn(T) -> ScalarValue,
    validate_constraints: fn(&T::Constraints, &T) -> Result<(), ConstraintViolation>,
    as_parameter_constraints: fn(&T::Constraints) -> ParameterConstraints,

    // --- journals ---
    journal_value: JournalHandle<ConfigPropertyValueRecord>,
    journal_state: JournalHandle<ConfigPropertyStateRecord>,
}

impl<T: PropertyType> ConfigProperty<T> {
    pub(crate) fn new(
        handle: ConfigPropertyHandle<T>,
        into_scalar: fn(T) -> ScalarValue,
        validate_constraints: fn(&T::Constraints, &T) -> Result<(), ConstraintViolation>,
        as_parameter_constraints: fn(&T::Constraints) -> ParameterConstraints,
        journal_value: JournalHandle<ConfigPropertyValueRecord>,
        journal_state: JournalHandle<ConfigPropertyStateRecord>,
    ) -> Self {
        Self {
            handle,
            into_scalar,
            validate_constraints,
            as_parameter_constraints,
            journal_value,
            journal_state,
        }
    }

    pub fn get_ref(&self) -> &T {
        self.validate();
        unsafe { self.handle.p_value.as_ref() }
    }

    pub fn set(&mut self, value: T) -> Result<bool, ConfigPropertyWriteError> {
        self.validate();

        if value == *self.get_ref() {
            return Ok(false);
        }

        let res = self.write(value.clone());
        let descriptor = self.descriptor();

        self.journal_value.append(ConfigPropertyValueRecord {
            ident: descriptor.ident,
            path: descriptor.resource.to_string(),
            value: (self.into_scalar)(value),
            origin: OperationOrigin::Machine,
            result: res.clone(),
            timestamp: Utc::now(),
        });

        res.map(|_| true)
    }

    /// resets property back to the assigned default value
    pub fn reset(&mut self) -> Result<bool, ConfigPropertyWriteError> {
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

        state.writable = value.clone();

        let descriptor = self.descriptor();
        self.journal_state.append(ConfigPropertyStateRecord {
            ident: descriptor.ident,
            path: descriptor.resource.to_string(),
            kind: ConfigPropertyStateChange::WriteCapability(value),
            timestamp: Utc::now(),
        });
    }

    pub fn set_default(&mut self, value: T) {
        self.validate();
        let state = unsafe { self.handle.p_state.as_mut() };

        if value == state.default_value {
            return;
        }

        let descriptor = self.descriptor();
        state.default_value = value.clone();

        self.journal_state.append(ConfigPropertyStateRecord {
            ident: descriptor.ident,
            path: descriptor.resource.to_string(),
            kind: ConfigPropertyStateChange::DefaultValue((self.into_scalar)(value)),
            timestamp: Utc::now(),
        });
    }

    // TODO: don't let strings mutate constraints !!!
    pub fn set_constraints(&mut self, value: T::Constraints) {
        self.validate();

        let state = unsafe { self.handle.p_state.as_mut() };

        if value == state.constraints {
            return;
        }

        state.constraints = value.clone();

        let descriptor = self.descriptor();
        let constraints = (self.as_parameter_constraints)(&state.constraints);

        self.journal_state.append(ConfigPropertyStateRecord {
            ident: descriptor.ident,
            path: descriptor.resource.to_string(),
            kind: ConfigPropertyStateChange::Constraints(constraints),
            timestamp: Utc::now(),
        });
    }

    // --- utils ---
    fn descriptor(&self) -> PropertyDescriptor {
        unsafe { self.handle.p_descriptor.read() }
    }

    fn write(&mut self, value: T) -> ConfigPropertyWriteResult {
        unsafe {
            let state = self.handle.p_state.as_ref();

            if let WriteCapability::Forbidden { .. } = state.writable {
                return Err(ConfigPropertyWriteError::NotWritable);
            };

            if let Err(e) = (self.validate_constraints)(&state.constraints, &value) {
                return Err(ConfigPropertyWriteError::ConstraintViolation(e));
            }

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
}

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
    value: *mut (),

    /// type-erased property state
    state: *const (),

    /// function to write scalar value into the property
    write: fn(*const (), ScalarValue, *mut ()) -> ConfigPropertyWriteResult,

    /// callback invoked when a external write succeeds and the value changed
    on_external_changed: Option<OnExternalChangedCallback>,
}

impl ExecuteContext {
    pub fn execute(
        self,
        machine: &mut dyn Machine,
        value: ScalarValue,
    ) -> ConfigPropertyWriteResult {
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

/// bump allocation based registry
pub struct ConfigPropertyRegistry {
    machines: heapless::Vec<MachineInfo, 64>,

    // --- item ---
    pool_items_pos: usize,
    pool_slot: Box<[SlotInfo]>,
    pool_info: Box<[MaybeUninit<PropertyDescriptor>]>,
    pool_exec: Box<[MaybeUninit<ExecuteContext>]>,
    pool_dirty: Box<[MaybeUninit<bool>]>,

    alloc_value: BumpAllocator,
    alloc_cache: BumpAllocator,
    alloc_state: BumpAllocator,
}

/// Safety mechanism for ensuring invalid handles cannot accidentaly write
#[derive(Clone, Copy, Default)]
pub struct SlotInfo {
    state: SlotState,
    generation: u64,
}

impl ConfigPropertyRegistry {
    pub fn new(pool_size: usize, items_max: usize) -> Self {
        Self {
            machines: heapless::Vec::new(),

            pool_items_pos: 0,
            pool_slot: vec![SlotInfo::default(); items_max].into_boxed_slice(),
            pool_info: vec![MaybeUninit::uninit(); items_max].into_boxed_slice(),
            pool_dirty: vec![MaybeUninit::uninit(); items_max].into_boxed_slice(),
            pool_exec: vec![MaybeUninit::uninit(); items_max].into_boxed_slice(),

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
            self.pool_slot[index].state = SlotState::Deactivated;
            self.pool_slot[index].generation += 1;
        }
    }

    pub fn execute_context(
        &self,
        ident: MachineIdentificationUnique,
        resource: &str,
    ) -> Option<ExecuteContext> {
        let machine_info = self.machines.iter().find(|m| m.ident == ident)?;

        for i in machine_info.pos..machine_info.len {
            unsafe {
                let info = self.pool_info[i].assume_init_read();

                if info.resource == resource {
                    return Some(self.pool_exec[i].assume_init_read());
                }
            }
        }

        None
    }

    pub fn begin_registration(
        &'_ mut self,
        ident: MachineIdentificationUnique,
    ) -> ConfigPropertyRegistryRegisterHandle<'_> {
        let item_pos = self.pool_items_pos;
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
        write: fn(*const (), ScalarValue, *mut ()) -> ConfigPropertyWriteResult,
        on_changed: Option<OnExternalChangedCallback>,
    ) -> ConfigPropertyHandle<T> {
        let index = self.registry.pool_items_pos;

        assert!(
            index < self.registry.pool_info.len(),
            "Property registry exhausted"
        );

        let p_value = self.registry.alloc_value.allocate::<T>();
        let p_cache = self.registry.alloc_cache.allocate::<T>();
        let p_state = self
            .registry
            .alloc_state
            .allocate::<ConfigPropertyState<T>>();

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
            ident: self.ident,
            resource: path,
            p_value: p_value.as_ptr() as *mut (),
            p_cache: p_cache.as_ptr() as *mut (),
            p_state: p_state.as_ptr() as *mut (),
        };

        let exec_ctx = ExecuteContext {
            value: p_value.as_ptr() as *mut (),
            state: p_state.as_ptr() as *const (),
            write,
            on_external_changed: on_changed,
        };

        self.registry.pool_info[index].write(descriptor);
        self.registry.pool_exec[index].write(exec_ctx);
        self.registry.pool_dirty[index].write(false);
        self.registry.pool_items_pos += 1;

        let p_descriptor = unsafe {
            NonNull::new_unchecked(
                self.registry
                    .pool_info
                    .as_mut_ptr()
                    .add(index)
                    .cast::<PropertyDescriptor>(),
            )
        };

        let p_slot = unsafe {
            NonNull::new_unchecked(
                &mut self.registry.pool_slot[index] as *mut SlotInfo,
            )
        };

        ConfigPropertyHandle {
            generation: self.registry.pool_slot[index].generation,
            p_slot,
            p_descriptor,
            p_state,
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

impl Drop for ConfigPropertyRegistryRegisterHandle<'_> {
    fn drop(&mut self) {
        if self.committed {
            return;
        }

        // --- roll back item count ---
        self.registry.pool_items_pos = self.item_pos;

        // mark all reserved slots as unused again and increment generation
        // so any created handles cannot access the memory
        for index in self.item_pos..self.registry.pool_items_pos {
            let slot_info = &mut self.registry.pool_slot[index];
            slot_info.state = SlotState::Unused;
            slot_info.generation += 1;
        }

        // Revert the bump allocations
        self.registry.alloc_value.rollback(self.value_mark);
        self.registry.alloc_cache.rollback(self.cache_mark);
        self.registry.alloc_state.rollback(self.state_mark);
    }
}
