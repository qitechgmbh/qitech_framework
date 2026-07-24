use std::borrow::Cow;

use chrono::Utc;
use qitech_framework_common::MachineConfigMutation;
use qitech_framework_common::MachineIdentificationUnique;
use qitech_framework_common::OperationOrigin;
use qitech_framework_common::OperationResult;
use qitech_framework_common::ScalarValue;
use serde::Serialize;
use serde::de::DeserializeOwned;

use super::JournalHandle;
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

pub struct ConfigProperty<T: Clone> {
    handle: PropertyHandle<T>,
    journal: JournalHandle<MachineConfigMutation>,
    validate: ValidateAndRecord<T>,
    default: T,
}

impl<T: Clone> ConfigProperty<T> {
    /// reset property back to default value
    pub fn reset(&mut self) {
        self.handle.write(self.default.clone());
    }
}

impl<T: Clone> ConfigProperty<T> {
    pub fn set(&mut self, value: T) -> Result<(), WriteError> {
        (self.validate)(&mut self.journal, &value)?;
        self.handle.write(value);
        Ok(())
    }
}

impl<T: Clone + Copy> ConfigProperty<T> {
    pub fn get(&self) -> T {
        *self.handle.read()
    }
}

// impl ConfigProperty<String> {
//     pub fn get(&self) -> &str { self.handle.read() }
// }
//
// impl ConfigProperty<String> {
//     pub fn get(&self) -> &str { self.handle.read() }
// }

/*
// uom impl
macro_rules! impl_uom {
    ($quantity:path, $unit:path, $unit_trait:path, $conversion_trait:path) => {
        impl ConfigProperty<$unit> {
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
                self.set(<$quantity>::new::<N>(value));
            }
        }

        impl ConfigProperty<Option<$unit>> {
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
                self.set(value.map(<$quantity>::new::<N>));
            }
        }

        impl ConstrainedConfigProperty<$unit> {
            pub fn get_as<N>(&self) -> f64
            where
                N: $unit_trait + $conversion_trait,
            {
                self.get().get::<N>()
            }

            pub fn set_as<N>(&mut self, value: f64) -> Result<(), String>
            where
                N: $unit_trait + $conversion_trait,
            {
                self.set(<$quantity>::new::<N>(value))
            }
        }

        impl ConstrainedConfigProperty<Option<$unit>> {
            pub fn get_as<N>(&self) -> Option<f64>
            where
                N: $unit_trait + $conversion_trait,
            {
                self.get().map(|q| q.get::<N>())
            }

            pub fn set_as<N>(&mut self, value: Option<f64>) -> Result<(), String>
            where
                N: $unit_trait + $conversion_trait,
            {
                self.set(value.map(<$quantity>::new::<N>))
            }
        }
    };
}

with_uom!(impl_uom);
*/

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
    pub(crate) fn unregister_machine(&mut self, ident: MachineIdentificationUnique) -> usize {
        self.registry.unregister_machine(ident)
    }
}

pub struct Metadata {
    write_api: WriteApiFn,
}

// --- registrar ---
pub struct Registrar<'a> {
    manager: &'a mut Manager,
    machine: MachineIdentificationUnique,
}

impl Registrar<'_> {
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
        /*
        // Validates `value` against `options`, returning the failure reason
        // (if any) without touching the journal.
        fn check<T>(value: &T, options: &RegisterOptions<T>) -> Result<(), CheckError> {
            value.validate(options.min, options.max).map_err(CheckError::OutOfBounds)?;
            if let Some(func) = options.validate {
                (func)(value).map_err(CheckError::Validate)?;
            }
            Ok(())
        }

        // used by the property itself
        let validate_and_record = Box::new(
            move |journal: &mut JournalHandle<MachineConfigMutation>,
                  value: &T::Type|
                  -> Result<(), WriteError> {
                if let Err(e) = check(value, &options) {
                    let entry = MachineConfigMutation {
                        target: ident,
                        resource_path: Cow::Borrowed(path),
                        value: T::into_string(value),
                        origin: OperationOrigin::Machine,
                        result: OperationResult::Failure,
                        timestamp: Utc::now(),
                    };
                    journal.append(entry).map_err(|_| WriteError::JournalFull)?;
                    return Err(e.into());
                }
                Ok(())
            },
        );

        let write_api = Box::new(
            move |request_id: u64,
                  journal: &mut Journal<MachineConfigMutation>,
                  bytes: *mut u8,
                  raw: &str|
                  -> Result<(), ApiWriteError> {
                let mut entry = MachineConfigMutation {
                    target: ident,
                    resource_path: Cow::Borrowed(path),
                    value: raw.to_string(),
                    origin: OperationOrigin::Request { request_id },
                    result: OperationResult::Success,
                    timestamp: Utc::now(),
                };

                let record_failure = |journal: &mut Journal<MachineConfigMutation>, entry: MachineConfigMutation| {
                    journal.init_handle().append(entry).map_err(|_| ApiWriteError::JournalFull)
                };

                let value: T::Type = match serde_json::from_str(raw) {
                    Ok(v) => v,
                    Err(e) => {
                        entry.result = OperationResult::Failure;
                        record_failure(journal, entry)?;
                        return Err(ApiWriteError::ParseError(e));
                    }
                };

                if let Err(e) = check(&value, &options) {
                    entry.result = OperationResult::Failure;
                    record_failure(journal, entry)?;
                    return Err(e.into());
                }

                record_failure(journal, entry)?;

                unsafe {
                    *(bytes as *mut T::Type) = value.clone();
                }

                Ok(())
            },
        );

        let metadata = Metadata { write_api };
        let handle = self
            .manager
            .registry
            .register::<T::Type>(ident, path, "", T::extract, metadata)?;

        handle.write(options.default.clone());

        Ok(ConfigProperty {
            handle,
            journal: self.manager.journal.init_handle(),
            validate: validate_and_record,
            default: options.default,
        })
        */
        todo!()
    }
}

#[derive(Debug, Default)]
pub struct RegisterOptions<T: BoundedMeta> {
    default: T,
    min: Option<T::Bound>,
    max: Option<T::Bound>,
    validate: Option<fn(&T) -> Result<(), String>>,
}

// --- types ---
pub type ValidateAndRecord<T> = Box<
    dyn Fn(&mut JournalHandle<MachineConfigMutation>, &T) -> Result<(), WriteError>
>;

pub type WriteApiFn = Box<
    dyn Fn(u64, &mut Journal<MachineConfigMutation>, *mut u8, &str) -> Result<(), ApiWriteError>,
>;

// --- errors ---
pub enum ApiWriteError {
    JournalFull,
    ParseError(serde_json::Error),
    ValueOutOfBounds(BoundsError),
    ValidateError(String),
}

pub enum WriteError {
    JournalFull,
    ValueOutOfBounds(BoundsError),
    ValidateError(String),
}
