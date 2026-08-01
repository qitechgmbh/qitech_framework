use std::any;
use std::any::Any;
use std::collections::HashMap;

use chrono::Utc;
use qitech_framework_core::ScalarValue;
use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::report::MachineConfigCapabilityMutation;
use qitech_framework_core::report::MachineConfigConstraints;
use qitech_framework_core::report::MachineConfigValueMutation;
use qitech_framework_core::report::MachineConfigWriteCapability;
use qitech_framework_core::report::MachineConfigWriteError;
use qitech_framework_core::report::OperationOrigin;
use qitech_framework_core::report::OperationResult;
use qitech_framework_core::with_uom_quantities;

use super::PropertyHandle;
use crate::machine::Machine;
use crate::machine::TypeWrapper;
use crate::machine::resource::Journal;
use crate::machine::resource::Key;
use crate::machine::resource::PropertyManager;
use crate::machine::resource::conversion::BoundedMeta;
use crate::machine::resource::error::RegisterResult;
use crate::machine::resource::error::ResourceAccessError;
use crate::machine::resource::subscription::SubscribeError;
use crate::machine::resource::subscription::SubscribedProperty;

pub struct ConfigProperty<T: Clone> {
    handle: PropertyHandle<T>,
    write: WriteFn,
    default: T,
}

impl<T: Clone> ConfigProperty<T> {
    pub fn set(&mut self, value: T) -> Result<(), MachineConfigWriteError> {

        self.
        (self.write)(self.handle.clone(), value)
    }

    /// reset property back to default value
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
    inner: PropertyManager<SLOT_SIZE, MAX_ITEMS, Kind, ()>,
    entries: HashMap<Key<'static>, Entry>,
    journal_value: Journal<MachineConfigValueMutation>,
    journal_capability: Journal<MachineConfigCapabilityMutation>,
}

impl Manager {
    pub fn register<T>(
        &mut self,
        machine: MachineIdentificationUnique,
        path: &'static str,
        options: RegisterOptions<T::Type>,
    ) -> RegisterResult<ConfigProperty<T::Type>>
    where
        T: TypeWrapper + 'static,
        T::Type: Clone + BoundedMeta,
    {
        // --- create handle ---
        let default = options.default.clone();
        let handle =
            self.inner
                .register::<T::Type>(machine, path.to_string(), (), default.clone())?;

        // --- internal writer ---
        let opts = options.clone();
        let journal = self.journal_value.new_handle();

        // --- internal writer ---
        let opts = options.clone();
        let journal = self.journal_value.new_handle();

        let write = Box::new(
            move |handle: PropertyHandle<T::Type>, value: T::Type| -> Result<(), WriteError> {
                let mut entry = MachineConfigValueMutation {
                    machine,
                    path: path.to_string(),
                    value: T::into_scalar(&value),
                    origin: OperationOrigin::Machine,
                    result: OperationResult::Failure,
                    timestamp: Utc::now(),
                };

                if let Err(e) = check(&value, &opts) {
                    journal.append(entry);
                    return Err(e);
                }

                entry.result = OperationResult::Success;
                journal.append(entry);
                handle.write(value);
                Ok(())
            },
        );

        // --- api writer ---
        let opts = options.clone();
        let journal = self.journal_value.new_handle();

        let handle_api = handle.clone();
        let write_api = Box::new(
            move |request_id: u64, value: ScalarValue| -> Result<(), ApiWriteError> {
                let Some(value) = T::from_scalar(value) else {
                    return Err(ApiWriteError::InvalidType);
                };

                let mut entry = MachineConfigValueMutation {
                    machine,
                    path: path.to_string(),
                    value: T::into_scalar(&value),
                    origin: OperationOrigin::Request { request_id },
                    result: OperationResult::Failure,
                    timestamp: Utc::now(),
                };

                if let Err(e) = check(&value, &opts) {
                    journal.append(entry);
                    return Err(match e {
                        WriteError::OutOfBounds(e) => ApiWriteError::OutOfBounds(e),
                        WriteError::RegexFailed(e) => ApiWriteError::Validate(e),
                    });
                }

                entry.result = OperationResult::Success;
                journal.append(entry);

                handle_api.write(value);
                Ok(())
            },
        );

        // --- store the api writer ---
        let key = Key::from_str(machine, path);
        self.entries.insert(
            key,
            Entry {
                fixed_constraints: (),
                get_capabilities: (),
            },
        );

        // --- create entry ---

        // --- create property ---
        let mut property = ConfigProperty {
            handle,
            write,
            default,
        };

        // invoke a reset to record the initial value
        property.reset();
        Ok(property)
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

    pub fn get_capabilities(
        &self,
        machine: MachineIdentificationUnique,
        path: &str,
        machine_ref: &dyn Machine,
    ) -> Result<MachineConfigPropertyCapabilities, ResourceAccessError> {
        let key = Key::from_str(machine, path);

        let Some(Entry {
            fixed_constraints,
            get_capabilities,
        }) = self.entries.get(&key)
        else {
            return Err(ResourceAccessError::NoSuchResource);
        };

        let dynamic = match get_capabilities {
            Some(get) => get(machine_ref)?,

            None => MachineConfigPropertyCapabilities {
                writable: MachineConfigWriteCapability {
                    disabled_reason: None,
                },
                constraints: MachineConfigConstraints::None,
            },
        };

        Ok(MachineConfigPropertyCapabilities {
            writable: dynamic.writable,
            constraints: fixed_constraints.merged(&dynamic.constraints),
        })
    }

    pub fn write(
        &mut self,
        target: MachineIdentificationUnique,
        path: &str,
        machine_ref: &dyn Machine,
        value: ScalarValue,
    ) -> Result<(), MachineConfigWriteError> {
        let key = Key::from_str(target, path);

        let Some(entry) = self.entries.get(&key) else {
            return Err(MachineConfigWriteError::NotFound);
        };

        let capabilities = {
            let dynamic = match &entry.get_capabilities {
                Some(get) => get(machine_ref)
                    .map_err(MachineConfigWriteError::MachineTypeMismatch)?,

                None => MachineConfigPropertyCapabilities {
                    write: WriteCapability::writable(),
                    constraints: MachineConfigConstraints::None,
                },
            };

            MachineConfigPropertyCapabilities {
                write: dynamic.write,
                constraints: entry.fixed_constraints.merged(&dynamic.constraints),
            }
        };

        if let Some(reason) = capabilities.write.disabled_reason {
            return Err(MachineConfigWriteError::NotWritable(reason));
        }

        let Some(Entry { fixed_constraints, get_capabilities }) = self.entries.get(&key) else {
            return Err(MachineConfigWriteError::NotWritable(None));
        };

        writer(value)
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
    fixed_constraints: MachineConfigConstraints,
    get_capabilities: GetCapabilitiesFn,
    write_value: WriteFn,
}

pub struct MachineConfigPropertyCapabilities {
    pub writable: MachineConfigWriteCapability,
    pub constraints: MachineConfigConstraints,
}

#[derive(Default)]
pub struct RegisterOptions<T: BoundedMeta> {
    pub default: T,
    pub fixed_constraints: MachineConfigConstraints,
    pub get_constraints: Option<GetCapabilitiesFn>,
}

// --- get capabilities ---
pub type GetCapabilitiesFn =
    Box<dyn Fn(&dyn Machine) -> Result<MachineConfigPropertyCapabilities, ResourceAccessError>>;

pub trait IntoGetConstraintsFn {
    fn into_get_constraints_fn(self) -> GetCapabilitiesFn;
}

impl<M> IntoGetConstraintsFn for fn(&M) -> MachineConfigPropertyCapabilities
where
    M: Machine + 'static,
{
    fn into_get_constraints_fn(self) -> GetCapabilitiesFn {
        Box::new(move |machine: &dyn Machine| {
            let machine = (machine as &dyn Any)
                .downcast_ref::<M>()
                .ok_or(ResourceAccessError::MachineTypeMismatch)?;

            Ok(self(machine))
        })
    }
}

// --- write fn ---
pub type WriteFn = Box<dyn Fn(ScalarValue) -> Result<(), MachineConfigWriteError>>;
