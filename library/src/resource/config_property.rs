use std::any::TypeId;
use std::mem::MaybeUninit;
use std::ptr::NonNull;

use chrono::Utc;
use qitech_framework_core::ScalarValue;
use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::report::ConfigPropertyStateMutationKind;
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
use crate::resource::OnChangedCallback;
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

        let descriptor = unsafe { self.handle.p_descriptor.read() };
        _ = descriptor;

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

    fn write(&mut self, value: T) -> ConfigPropertyWriteResult {
        self.validate();

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

    /// resets property back to the assigned default value
    pub fn reset(&mut self) -> ConfigPropertyWriteResult {
        self.validate();

        unsafe {
            let default = &self.handle.p_state.as_ref().default_value;
            self.write(default.clone())
        }
    }

    pub fn set_writable(&mut self, value: WriteCapability) {
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
            kind: ConfigPropertyStateMutationKind::WriteCapability(value),
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
        assert_eq!(descriptor.state, SlotState::Activated);

        state.default_value = value.clone();

        self.journal_state.append(ConfigPropertyStateRecord {
            ident: descriptor.ident,
            path: descriptor.resource.to_string(),
            kind: ConfigPropertyStateMutationKind::DefaultValue((self.into_scalar)(value)),
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
            kind: ConfigPropertyStateMutationKind::Constraints(constraints),
            timestamp: Utc::now(),
        });
    }

    fn descriptor(&self) -> PropertyDescriptor {
        unsafe { self.handle.p_descriptor.read() }
    }

    /// ensures that the slot is activated, aka we have permission to write
    fn validate(&self) {
        assert_eq!(self.descriptor().state, SlotState::Activated);
    }
}

pub struct ConfigPropertyHandle<T: PropertyType> {
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
    on_changed: Option<OnChangedCallback>,
}

impl ExecuteContext {
    pub fn execute(
        self,
        machine: &mut dyn Machine,
        value: ScalarValue,
    ) -> ConfigPropertyWriteResult {
        let result = (self.write)(self.state, value, self.value);

        if let Some(callback) = self.on_changed
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
    pool_info: Box<[MaybeUninit<PropertyDescriptor>]>,
    pool_exec: Box<[MaybeUninit<ExecuteContext>]>,
    pool_dirty: Box<[MaybeUninit<bool>]>,

    alloc_value: BumpAllocator,
    alloc_cache: BumpAllocator,
    alloc_state: BumpAllocator,
}

impl ConfigPropertyRegistry {
    pub fn new(pool_size: usize, items_max: usize) -> Self {
        Self {
            machines: heapless::Vec::new(),

            pool_items_pos: 0,
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

        for index in machine.pos..machine.pos + machine.len {
            unsafe {
                self.pool_info[index].assume_init_mut().state = SlotState::Deactivated;
            }
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

    pub fn begin_commit(
        &'_ mut self,
        ident: MachineIdentificationUnique,
    ) -> ConfigPropertyRegistryCommitHandle<'_> {
        let item_pos = self.pool_items_pos;
        let value_mark = self.alloc_value.mark();
        let cache_mark = self.alloc_cache.mark();
        let state_mark = self.alloc_state.mark();

        ConfigPropertyRegistryCommitHandle {
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

pub struct ConfigPropertyRegistryCommitHandle<'a> {
    registry: &'a mut ConfigPropertyRegistry,
    ident: MachineIdentificationUnique,
    item_pos: usize,
    value_mark: BumpAllocatorMark,
    cache_mark: BumpAllocatorMark,
    state_mark: BumpAllocatorMark,
    committed: bool,
}

impl<'a> ConfigPropertyRegistryCommitHandle<'a> {
    pub fn register<T: PropertyType>(
        &mut self,
        path: &'static str,
        default: T,
        writable: WriteCapability,
        constraints: T::Constraints,
        write: fn(*const (), ScalarValue, *mut ()) -> ConfigPropertyWriteResult,
        on_changed: Option<OnChangedCallback>,
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
            state: SlotState::Reserved,
            type_id: TypeId::of::<T>(),
            ident: self.ident,
            resource: path,
            p_value: p_value.as_ptr() as *mut (),
            p_cache: p_cache.as_ptr() as *mut (),
            p_state: p_state.as_ptr() as *mut (),
        };

        let exec_ctx = ExecuteContext {
            value: p_value.as_ptr() as *mut (),
            state: p_state.as_ptr() as *mut (),
            write,
            on_changed,
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

        ConfigPropertyHandle {
            p_descriptor,
            p_state,
            p_value,
        }
    }

    pub fn commit(mut self) {
        for index in self.item_pos..self.registry.pool_items_pos {
            let descriptor = unsafe { self.registry.pool_info[index].assume_init_mut() };
            descriptor.state = SlotState::Activated;
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

impl Drop for ConfigPropertyRegistryCommitHandle<'_> {
    fn drop(&mut self) {
        if self.committed {
            return;
        }

        // Revert the bump allocations
        self.registry.alloc_value.rollback(self.value_mark);
        self.registry.alloc_cache.rollback(self.cache_mark);
        self.registry.alloc_state.rollback(self.state_mark);
    }
}
