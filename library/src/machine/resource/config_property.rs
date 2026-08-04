use std::any::Any;
use std::borrow::Cow;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ptr::NonNull;
use std::rc::Rc;

use chrono::Utc;
use qitech_framework_core::ScalarValue;
use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::report::ConstraintViolation;
use qitech_framework_core::report::MachineConfigCapabilityMutation;
use qitech_framework_core::report::MachineConfigPropertyConstraints;
use qitech_framework_core::report::MachineConfigValueMutation;
use qitech_framework_core::report::MachineConfigWriteCapability;
use qitech_framework_core::report::MachineConfigWriteError;
use qitech_framework_core::report::OperationOrigin;
use qitech_framework_core::with_uom_quantities;

use super::PropertyHandle;
use crate::machine::Machine;
use crate::machine::TypeWrapper;
use crate::machine::resource::Journal;
use crate::machine::resource::Key;
use crate::machine::resource::PropertyManager;
use crate::machine::resource::error::RegisterResult;
use crate::machine::resource::error::ResourceAccessError;
use crate::machine::resource::subscription::SubscribeError;
use crate::machine::resource::subscription::SubscribedProperty;

pub struct ConfigProperty<T: Clone> {
    handle: PropertyHandle<T>,
    default: T,
    apply: ApplyFn<T>,
}

impl<T: Clone> ConfigProperty<T> {
    pub fn set(&mut self, value: T) -> Result<(), MachineConfigWriteError> {
        (self.apply)(value)
    }

    pub fn reset(&mut self) {
        self.set(self.default.clone())
            .expect("Default must pass validation");
    }

    pub fn get_ref(&self) -> &T {
        self.handle.read()
    }
}

impl<T: Copy> ConfigProperty<T> {
    pub fn get(&self) -> T {
        *self.handle.read()
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

            pub fn set_as<N>(&mut self, value: f64) -> Result<(), MachineConfigWriteError>
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

            pub fn set_as<N>(&mut self, value: Option<f64>) -> Result<(), MachineConfigWriteError>
            where
                N: $unit_trait + $conversion_trait,
            {
                self.set(value.map(<$quantity>::new::<N>))
            }
        }
    };
}

with_uom_quantities!(impl_uom);

// --- resource managment ---
const SLOT_SIZE: usize = 32;
const MAX_ITEMS: usize = 512;
type Kind = super::property_kind::ConfigProperty;

#[derive(Default)]
pub struct Manager {
    inner: PropertyManager<SLOT_SIZE, MAX_ITEMS, Kind>,
    entries: HashMap<Key<'static>, Entry>,
    journal_value: Journal<MachineConfigValueMutation>,
    journal_capability: Journal<MachineConfigCapabilityMutation>,
}

impl Manager {
    // --- registration ---
    pub fn register<T: TypeWrapper>(
        &mut self,
        ident: MachineIdentificationUnique,
        path: &'static str,
        default: T::Type,
        get_capabilities: GetCapabilitiesFn,
        on_changed: OnChangedFn,
    ) -> RegisterResult<ConfigProperty<T::Type>> {
        // --- create handle ---
        let handle =
            self.inner
                .register::<T::Type>(ident, path.to_string(), (), default.clone())?;

        // --- writing ---
        let journal_capability = self.journal_capability.new_handle();

        // initialize shared capabilities
        let current_capabilities = Rc::new(RefCell::new(ConfigPropertyCapabilitiesAny {
            writable: MachineConfigWriteCapability::Allowed,
            constraints: MachineConfigPropertyConstraints::None,
        }));

        let write_get_capabilities = get_capabilities.clone();
        let write_handle = handle.clone();
        let write_current_capabilities = current_capabilities.clone();

        let write_fn: WriteFn = Rc::new(move |machine: &mut dyn Machine, value: ScalarValue| {
            let capabilities = match (write_get_capabilities)(machine) {
                Ok(v) => v,

                Err(ResourceAccessError::NoSuchResource) => {
                    return Err(MachineConfigWriteError::NotFound);
                }

                Err(ResourceAccessError::NoSuchMachine) => {
                    return Err(MachineConfigWriteError::MachineNotFound);
                }

                Err(ResourceAccessError::MachineTypeMismatch) => {
                    return Err(MachineConfigWriteError::MachineTypeMismatch);
                }
            };

            let can_write = capabilities.writable == MachineConfigWriteCapability::Allowed;
            let validate_result = validate_constraints(&value, &capabilities.constraints);

            if capabilities != *write_current_capabilities.borrow() {
                journal_capability.append(MachineConfigCapabilityMutation {
                    ident,
                    path: path.to_string(),
                    writable: capabilities.writable.clone(),
                    constraints: capabilities.constraints.clone(),
                    timestamp: Utc::now(),
                });

                *write_current_capabilities.borrow_mut() = capabilities;
            }

            validate_result?;

            if !can_write {
                return Err(MachineConfigWriteError::NotWritable);
            }

            let value = T::from_scalar(value).ok_or(MachineConfigWriteError::ValueTypeMismatch)?;

            write_handle.write(value);

            if let Err(e) = (on_changed)(machine) {
                return Err(MachineConfigWriteError::CallbackFailure(e));
            }

            Ok(())
        });

        let machine_ref: Rc<Cell<Option<NonNull<dyn Machine>>>> = Rc::new(Cell::new(None));

        // --- recording ---
        let journal_value = self.journal_value.new_handle();

        let record = Rc::new(
            move |value: T::Type, result: Result<(), MachineConfigWriteError>| {
                let entry = MachineConfigValueMutation {
                    ident,
                    path: path.to_string(),
                    value: T::into_scalar(value),
                    timestamp: Utc::now(),
                    origin: OperationOrigin::Machine,
                    result,
                };

                journal_value.append(entry);
            },
        );

        // --- apply ---
        let apply: ApplyFn<T::Type> = {
            let write_fn = write_fn.clone();
            let machine_ref = machine_ref.clone();

            Box::new(move |value: T::Type| {
                let mut machine_ptr = machine_ref
                    .get()
                    .expect("pointer must be initialized before machine runs");

                let machine = unsafe { machine_ptr.as_mut() };
                let value_scalar = T::into_scalar(value.clone());
                let result = write_fn(machine, value_scalar);
                record(value, result.clone());
                result
            })
        };

        let prop = ConfigProperty {
            handle,
            default: default.clone(),
            apply,
        };

        let entry = Entry {
            current_capabilities,
            get_capabilities,
            write_value: write_fn,
            machine_ref,
        };

        self.entries.insert(Key::from_str(ident, path), entry);

        self.journal_value
            .new_handle()
            .append(MachineConfigValueMutation {
                ident,
                path: path.to_string(),
                value: T::into_scalar(default),
                origin: OperationOrigin::Machine,
                result: Ok(()),
                timestamp: Utc::now(),
            });

        Ok(prop)
    }

    pub fn unregister_machine(&mut self, ident: MachineIdentificationUnique) {
        // --- remove property resources associated with machine ---
        self.inner.unregister_machine(ident);

        // --- remove api writers associated with machine ---
        self.entries.retain(|key, _| key.ident != ident);
    }

    // --- subscription ---
    pub fn create_subscriber<T: 'static>(
        &mut self,
        provider: MachineIdentificationUnique,
        subscriber: MachineIdentificationUnique,
        resource: &'static str,
    ) -> Result<SubscribedProperty<T>, SubscribeError> {
        self.inner.create_subscriber(provider, subscriber, resource)
    }

    pub fn remove_subscription(
        &mut self,
        provider: MachineIdentificationUnique,
        consumer: MachineIdentificationUnique,
    ) {
        self.inner.remove_subscription(provider, consumer);
    }

    pub fn sync_cache(&mut self) {
        self.inner.sync_cache();
    }

    pub fn write_value(
        &mut self,
        target: MachineIdentificationUnique,
        path: &str,
        machine: &mut dyn Machine,
        value: ScalarValue,
    ) -> Result<(), MachineConfigWriteError> {
        let key = Key {
            ident: target,
            path: Cow::Owned(path.to_string()),
        };

        let Some(Entry { write_value, .. }) = self.entries.get(&key) else {
            return Err(MachineConfigWriteError::NotFound);
        };

        let result = (write_value)(machine, value.clone());

        self.journal_value
            .new_handle()
            .append(MachineConfigValueMutation {
                ident: target,
                path: path.to_string(),
                value,
                origin: OperationOrigin::Machine,
                result: result.clone(),
                timestamp: Utc::now(),
            });

        result
    }

    /// read each properties capabilities and record changed ones
    pub fn sync_machines_capabilities(
        &mut self,
        machine: MachineIdentificationUnique,
        machine_ref: &dyn Machine,
    ) -> Result<(), ResourceAccessError> {
        Ok(())
    }

    // --- reporting ---
    pub fn drain_journal_value(&mut self, f: impl FnMut(MachineConfigValueMutation)) {
        self.journal_value.drain_with(f);
    }

    pub fn drain_journal_capability(&mut self, f: impl FnMut(MachineConfigCapabilityMutation)) {
        self.journal_capability.drain_with(f);
    }
}

pub struct Entry {
    current_capabilities: Rc<RefCell<ConfigPropertyCapabilitiesAny>>,
    get_capabilities: GetCapabilitiesFn,
    write_value: WriteFn,
    machine_ref: Rc<Cell<Option<NonNull<dyn Machine>>>>,
}

#[derive(PartialEq)]
pub struct ConfigPropertyCapabilities<T: TypeWrapper> {
    pub writable: MachineConfigWriteCapability,
    pub constraints: T::Constraints,
}

#[derive(Clone, PartialEq)]
pub struct ConfigPropertyCapabilitiesAny {
    pub writable: MachineConfigWriteCapability,
    pub constraints: MachineConfigPropertyConstraints,
}

// --- get capabilities ---
pub type GetCapabilitiesFn =
    Rc<dyn Fn(&dyn Machine) -> Result<ConfigPropertyCapabilitiesAny, ResourceAccessError>>;

pub trait IntoGetCapabilitiesFn {
    fn into_get_capabilities_fn(self) -> GetCapabilitiesFn;
}

impl<T, M> IntoGetCapabilitiesFn for fn(&M) -> ConfigPropertyCapabilities<T>
where
    T: TypeWrapper + 'static,
    M: Machine + 'static,
{
    fn into_get_capabilities_fn(self) -> GetCapabilitiesFn {
        Rc::new(move |machine: &dyn Machine| {
            let machine = (machine as &dyn Any)
                .downcast_ref::<M>()
                .ok_or(ResourceAccessError::MachineTypeMismatch)?;

            let capabilities = self(machine);

            Ok(ConfigPropertyCapabilitiesAny {
                writable: capabilities.writable,
                constraints: T::into_constraints(capabilities.constraints),
            })
        })
    }
}

// --- callback ---
pub type OnChangedFn = Box<dyn Fn(&mut dyn Machine) -> Result<(), String>>;

pub trait IntoOnChangedFn {
    fn into_on_changed_fn(self) -> OnChangedFn;
}

impl<M> IntoOnChangedFn for fn(&mut M) -> Result<(), String>
where
    M: Machine + 'static,
{
    fn into_on_changed_fn(self) -> OnChangedFn {
        Box::new(move |machine: &mut dyn Machine| {
            let machine = (machine as &mut dyn Any)
                .downcast_mut::<M>()
                .expect("write passed so this should never fail");

            (self)(machine)
        })
    }
}

// --- write fn ---
pub type WriteFn = Rc<dyn Fn(&mut dyn Machine, ScalarValue) -> Result<(), MachineConfigWriteError>>;

// --- apply ---
pub type ApplyFn<T> = Box<dyn Fn(T) -> Result<(), MachineConfigWriteError>>;

// --- utils ---
pub fn validate_constraints(
    value: &ScalarValue,
    constraints: &MachineConfigPropertyConstraints,
) -> Result<(), ConstraintViolation> {
    match (value, constraints) {
        // --- unconstrained: anything passes ---
        (_, MachineConfigPropertyConstraints::None) => Ok(()),

        // --- nullable variants ---
        (ScalarValue::Float(None), MachineConfigPropertyConstraints::Float { .. }) => Ok(()),
        (ScalarValue::Integer(None), MachineConfigPropertyConstraints::Integer { .. }) => Ok(()),
        (ScalarValue::String(None), MachineConfigPropertyConstraints::String { .. }) => Ok(()),
        (ScalarValue::Enum(None), MachineConfigPropertyConstraints::Enum { .. }) => Ok(()),

        // --- float ---
        (ScalarValue::Float(Some(v)), MachineConfigPropertyConstraints::Float { min, max }) => {
            if let Some(min) = min
                && v < min
            {
                return Err(ConstraintViolation::F64BelowMin {
                    value: *v,
                    min: *min,
                });
            }

            if let Some(max) = max
                && v > max
            {
                return Err(ConstraintViolation::F64AboveMax {
                    value: *v,
                    max: *max,
                });
            }

            Ok(())
        }

        // --- integer ---
        (ScalarValue::Integer(Some(v)), MachineConfigPropertyConstraints::Integer { min, max }) => {
            if let Some(min) = min
                && v < min
            {
                return Err(ConstraintViolation::I64BelowMin {
                    value: *v,
                    min: *min,
                });
            }

            if let Some(max) = max
                && v > max
            {
                return Err(ConstraintViolation::I64AboveMax {
                    value: *v,
                    max: *max,
                });
            }
            Ok(())
        }

        // --- string ---
        (
            ScalarValue::String(Some(v)),
            MachineConfigPropertyConstraints::String {
                min_length,
                max_length,
            },
        ) => {
            let length = v.chars().count();

            if let Some(min_length) = min_length
                && length < *min_length
            {
                return Err(ConstraintViolation::StringTooShort {
                    length,
                    min: *min_length,
                });
            }

            if let Some(max_length) = max_length
                && length > *max_length
            {
                return Err(ConstraintViolation::StringTooLong {
                    length,
                    max: *max_length,
                });
            }

            Ok(())
        }

        // --- enum ---
        (ScalarValue::Enum(Some(v)), MachineConfigPropertyConstraints::Enum { allowed }) => {
            if allowed.contains(v) {
                Ok(())
            } else {
                Err(ConstraintViolation::VariantForbidden { value: v.clone() })
            }
        }

        // --- everything else: value's variant doesn't match constraint's kind ---
        (_, _) => Err(ConstraintViolation::TypeMismatch),
    }
}

// --- testing ---
#[cfg(test)]
mod test {
    #[test]
    pub fn register_and_use() -> anyhow::Result<()> {
        /*
        let ident = MachineIdentificationUnique {
            identification: MachineIdentification {
                vendor_id: 0,
                machine_id: 0,
            },
            serial: 0,
        };

        let mut r = Manager::default();

        let mut prop = r.register::<f64>(
            ident,
            "just.some.config",
            RegisterOptions {
                default: 0.0,
                constraints: MachineConfigPropertyConstraints::None,
                get_constraints: None,
            },
        )?;

        prop.get_ref();
        prop.get();
        prop.set(10.0);
        prop.reset();
        */

        Ok(())
    }
}
