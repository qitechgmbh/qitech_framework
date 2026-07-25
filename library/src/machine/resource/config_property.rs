use std::borrow::Cow;

use chrono::Utc;
use qitech_framework_common::MachineConfigMutation;
use qitech_framework_common::MachineIdentificationUnique;
use qitech_framework_common::OperationOrigin;
use qitech_framework_common::OperationResult;
use qitech_framework_common::with_uom_quantities;
use serde::de::DeserializeOwned;

use super::PropertyHandle;
use crate::machine::error::BoundsError;
use crate::machine::resource::Journal;
use crate::machine::resource::PropertyManager;
use crate::machine::resource::PropertyReadHandle;
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
        (self.write)(&mut self.handle, value)
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
type Kind = super::property_kind::ConfigProperty;
type Metadata = WriteApiFn;

pub type RemoteHandle<T> = PropertyReadHandle<Kind, T>;

#[derive(Default)]
pub struct Manager {
    inner: PropertyManager<SLOT_SIZE, MAX_ITEMS, Kind, Metadata>,
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
        T::Type: Clone + DeserializeOwned + BoundedMeta,
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

        let default = options.default;
        let handle = self
            .inner
            .register::<T::Type>(ident, path, "", write_api, default.clone())?;

        Ok(ConfigProperty {
            handle,
            write,
            default,
        })
    }

    pub fn unregister_machine(&mut self, ident: MachineIdentificationUnique) {
        self.inner.unregister_machine(ident)
    }

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
