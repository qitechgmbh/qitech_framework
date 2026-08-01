use std::collections::HashMap;
use std::fmt::Debug;

use chrono::Utc;
use qitech_framework_core::MachineConfigMutation;
use qitech_framework_core::MachineIdentificationUnique;
use qitech_framework_core::OperationOrigin;
use qitech_framework_core::OperationResult;
use qitech_framework_core::with_uom_quantities;
use thiserror::Error;

use super::PropertyHandle;
use crate::machine::TypeWrapper;
use crate::machine::error::BoundsError;
use crate::machine::resource::Journal;
use crate::machine::resource::Key;
use crate::machine::resource::PropertyManager;
use crate::machine::resource::conversion::BoundedMeta;
use crate::machine::resource::error::RegisterResult;
use crate::machine::resource::subscription::SubscribeError;
use crate::machine::resource::subscription::SubscribedProperty;

pub struct ConfigProperty<T: Clone> {
    handle: PropertyHandle<T>,
    write: WriteFn<T>,
    default: T,
}

impl<T: Clone> ConfigProperty<T> {
    pub fn set(&mut self, value: T) -> Result<(), WriteError> {
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

            pub fn set_as<N>(&mut self, value: f64) -> Result<(), WriteError>
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

            pub fn set_as<N>(&mut self, value: Option<f64>) -> Result<(), WriteError>
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
    journal: Journal<MachineConfigMutation>,
    writers: HashMap<Key<'static>, WriteApiFn>,
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
        let journal = self.journal.new_handle();

        let write = Box::new(
            move |handle: PropertyHandle<T::Type>, value: T::Type| -> Result<(), WriteError> {
                let mut entry = MachineConfigMutation {
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
        let journal = self.journal.new_handle();

        let handle_api = handle.clone();
        let write_api = Box::new(
            move |request_id: u64,
                  raw: &str|
                  -> Result<Result<(), WriteError>, serde_json::Error> {
                let value: T::Type = T::deserialize_json(raw)?;

                let mut entry = MachineConfigMutation {
                    machine,
                    path: path.to_string(),
                    value: T::into_scalar(&value),
                    origin: OperationOrigin::Request { request_id },
                    result: OperationResult::Failure,
                    timestamp: Utc::now(),
                };

                if let Err(e) = check(&value, &opts) {
                    journal.append(entry);
                    return Ok(Err(e));
                }

                entry.result = OperationResult::Success;
                journal.append(entry);

                handle_api.write(value);
                Ok(Ok(()))
            },
        );

        // --- store the api writer ---
        let key = Key::from_str(machine, path);
        self.writers.insert(key, write_api);

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
        self.writers.retain(|key, _| key.ident != ident);
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

    // --- api ---
    pub fn api_write(
        &mut self,
        transaction_id: u64,
        target: MachineIdentificationUnique,
        path: &str,
        value: String,
    ) -> Result<(), ApiWriteError> {
        let key = Key::from_str(target, path);

        let Some(write) = self.writers.get(&key) else {
            return Err(ApiWriteError::NotFound);
        };

        let result = (write)(transaction_id, &value).map_err(ApiWriteError::ParseError)?;

        if let Err(e) = result {
            return Err(match e {
                WriteError::OutOfBounds(e) => ApiWriteError::OutOfBounds(e),
                WriteError::Validate(e) => ApiWriteError::Validate(e),
            });
        }

        Ok(())
    }

    // --- reporting ---
    pub fn drain_journal(&mut self, f: impl FnMut(MachineConfigMutation)) {
        self.journal.drain_with(f);
    }
}

#[derive(Debug, Clone, Default)]
pub struct RegisterOptions<T: BoundedMeta> {
    pub default: T,
    pub min: Option<T::Bound>,
    pub max: Option<T::Bound>,

    #[allow(clippy::type_complexity)]
    pub validate: Option<fn(&T) -> Result<(), String>>,
}

fn check<T: BoundedMeta>(value: &T, options: &RegisterOptions<T>) -> Result<(), WriteError> {
    // value
    //     .validate(options.min, options.max)
    //     .map_err(WriteError::OutOfBounds)?;
    //
    // if let Some(min) = options.min && value < min {
    //     return WriteError::OutOfBounds(Bound);
    // }
    // TODO: IMPLEMENT

    if let Some(func) = options.validate {
        (func)(value).map_err(WriteError::Validate)?;
    }

    Ok(())
}

// --- types ---
pub type WriteFn<T> = Box<dyn Fn(PropertyHandle<T>, T) -> Result<(), WriteError>>;
pub type WriteApiFn = Box<dyn Fn(u64, &str) -> Result<Result<(), WriteError>, serde_json::Error>>;

// --- errors ---
#[derive(Error, Debug)]
pub enum WriteError {
    #[error("value out of bounds: {0}")]
    OutOfBounds(BoundsError),

    #[error("validation failed: {0}")]
    Validate(String),
}

#[derive(Error, Debug)]
pub enum ApiWriteError {
    #[error("resource not found for machine")]
    NotFound,

    #[error("failed to parse value: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("value out of bounds: {0}")]
    OutOfBounds(BoundsError),

    #[error("validation failed: {0}")]
    Validate(String),
}
