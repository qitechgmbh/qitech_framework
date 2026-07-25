use std::borrow::Cow;

use chrono::Utc;
use qitech_framework_common::MachineConfigMutation;
use qitech_framework_common::MachineIdentificationUnique;
use qitech_framework_common::OperationOrigin;
use qitech_framework_common::OperationResult;
use qitech_framework_common::ScalarValue;
use qitech_framework_common::with_uom_quantities;
use serde::Serialize;
use serde::de::DeserializeOwned;

use super::PropertyHandle;
use crate::machine::error::BoundsError;
use crate::machine::resource::Journal;
use crate::machine::resource::PropertyAccessor;
use crate::machine::resource::PropertyReadHandle;
use crate::machine::resource::PropertyRegistry;
use crate::machine::resource::PropertyResolver;
use crate::machine::resource::conversion::BoundedMeta;
use crate::machine::resource::conversion::ScalarTypeWrapper;
use crate::machine::resource::error::RegisterError;
use crate::uom;

pub struct ConfigProperty<T: Clone> {
    handle: PropertyHandle<T>,
    write: WriteFn<T>,
    default: T,
}

impl<T: Clone> ConfigProperty<T> {
    pub fn set(&mut self, value: T) -> Result<(), WriteError> {
        (self.write)(&mut self.handle, value);
        Ok(())
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

with_uom_quantities!(uom, impl_uom);

// --- resource managment ---
const SLOT_SIZE: usize = 32;
const MAX_ITEMS: usize = 512;
type Kind = super::kind::ConfigProperty;

pub type Registry = PropertyRegistry<SLOT_SIZE, MAX_ITEMS, Kind, ScalarValue, Metadata>;
pub type Resolver<'a> = PropertyResolver<'a, SLOT_SIZE, MAX_ITEMS, Kind, ScalarValue, Metadata>;
pub type Accessor<'a> = PropertyAccessor<'a, SLOT_SIZE, MAX_ITEMS, Kind, ScalarValue, Metadata>;
pub type RemoteHandle<T> = PropertyReadHandle<Kind, T>;

pub struct Manager {
    registry: Registry,
    journal: Journal<MachineConfigMutation>,
}

impl Manager {
    pub fn register<T>(
        &mut self,
        ident: MachineIdentificationUnique,
        path: &'static str,
        options: RegisterOptions<T::Type>,
    ) -> Result<ConfigProperty<T::Type>, RegisterError>
    where
        T: ScalarTypeWrapper + 'static,
        T::Type: Clone + Serialize + DeserializeOwned + BoundedMeta,
    {
        let opts = options.clone();
        let journal = self.journal.new_handle();

        let write = Box::new(
            move |handle: &mut PropertyHandle<T::Type>, value: T::Type| -> Result<(), WriteError> {
                let mut entry = MachineConfigMutation {
                    target: ident,
                    resource_path: Cow::Borrowed(path),
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

        let opts = options.clone();
        let journal = self.journal.new_handle();

        let write_api = Box::new(
            move |request_id: u64,
                  bytes: *mut u8,
                  raw: &str|
                  -> Result<Result<(), WriteError>, serde_json::Error> {
                let value: T::Type = serde_json::from_str(raw)?;
                let out = unsafe { &mut *(bytes as *mut T::Type) };

                let mut entry = MachineConfigMutation {
                    target: ident,
                    resource_path: Cow::Borrowed(path),
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

                *out = value;
                Ok(Ok(()))
            },
        );

        let metadata = Metadata { write_api };

        let handle = self
            .registry
            .register::<T::Type>(ident, path, "", T::extract, metadata)?;

        let default = options.default;
        handle.write(default.clone());

        Ok(ConfigProperty {
            handle,
            write,
            default,
        })
    }

    pub(crate) fn unregister_machine(&mut self, ident: MachineIdentificationUnique) -> usize {
        self.registry.unregister_machine(ident)
    }
}

pub struct Metadata {
    write_api: WriteApiFn,
}

#[derive(Debug, Clone, Default)]
pub struct RegisterOptions<T: BoundedMeta> {
    default: T,
    min: Option<T::Bound>,
    max: Option<T::Bound>,
    validate: Option<fn(&T) -> Result<(), String>>,
}

fn check<T: BoundedMeta>(value: &T, options: &RegisterOptions<T>) -> Result<(), WriteError> {
    value
        .validate(options.min, options.max)
        .map_err(WriteError::OutOfBounds)?;

    if let Some(func) = options.validate {
        (func)(value).map_err(WriteError::Validate)?;
    }

    Ok(())
}

// --- types ---
pub type WriteFn<T> = Box<dyn Fn(&mut PropertyHandle<T>, T) -> Result<(), WriteError>>;

pub type WriteApiFn =
    Box<dyn Fn(u64, *mut u8, &str) -> Result<Result<(), WriteError>, serde_json::Error>>;

// --- errors ---
use core::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum WriteError {
    OutOfBounds(BoundsError),
    Validate(String),
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WriteError::OutOfBounds(err) => {
                write!(f, "value out of bounds: {}", err)
            }
            WriteError::Validate(msg) => {
                write!(f, "validation failed: {}", msg)
            }
        }
    }
}

impl Error for WriteError {}
