use chrono::Utc;
use serde::{Serialize, de::DeserializeOwned};
use std::{
    any::{self, Any},
    borrow::Cow,
    cell::RefCell,
    fmt::Debug,
    rc::Rc,
};

use qitech_framework_common::{
    MachineConfigMutation, OperationOrigin, OperationResult, ScalarValue,
};

use crate::machine::{
    Machine,
    build::conversion::{Extract, ScalarTypeWrapper, TypeWrapper},
    error::BoundsError,
    resource::{
        Journal, JournalHandle,
        command::{CommandExecuteError, CommandHandle},
        config_property::{ApiWriteConfigError, ConfigProperty, WriteConfigError},
        event::EventEmitter,
        measurement::Measurement,
        state_property::StateProperty,
    },
};

use super::{BuildContext, error::BuildError};

// --- config property ---
type ValidateConfigInputFn<T> = fn(&T) -> Result<(), String>;

#[derive(Debug, Default)]
pub struct ConfigPropertyOptions<T: BoundedMeta> {
    default: T,
    min: Option<T::Bound>,
    max: Option<T::Bound>,
    validate: Option<ValidateConfigInputFn<T>>,
}

impl<'a> BuildContext<'a> {
    pub fn create_config_property<T>(
        &mut self,
        path: &'static str,
        options: ConfigPropertyOptions<T::Type>,
    ) -> Result<ConfigProperty<T::Type>, BuildError>
    where
        T: ScalarTypeWrapper,
        T::Type: Clone + Serialize + DeserializeOwned + BoundedMeta,
    {
        let ident = self.ident;

        let write_api = Box::new(
            move |request_id: u64,
                  journal: Rc<RefCell<Journal<MachineConfigMutation>>>,
                  bytes: *mut u8,
                  value: &str|
                  -> Result<(), ApiWriteConfigError> {
                let mut entry = MachineConfigMutation {
                    target: ident,
                    resource_path: Cow::Borrowed(path),
                    value: value.to_string(),
                    origin: OperationOrigin::Request { request_id },
                    result: OperationResult::Success,
                    timestamp: Utc::now(),
                };

                let value: T::Type = match serde_json::from_str(value) {
                    Ok(v) => v,
                    Err(e) => {
                        entry.result = OperationResult::Failure;

                        journal
                            .borrow_mut()
                            .push(entry)
                            .map_err(|_| ApiWriteConfigError::JournalFull)?;

                        return Err(ApiWriteConfigError::ParseError(e));
                    }
                };

                if let Err(e) = value.validate(options.min, options.max) {
                    entry.result = OperationResult::Failure;

                    journal
                        .borrow_mut()
                        .push(entry)
                        .map_err(|_| ApiWriteConfigError::JournalFull)?;

                    return Err(ApiWriteConfigError::ValueOutOfBounds(e));
                };

                if let Some(func) = options.validate
                    && let Err(e) = (func)(&value)
                {
                    entry.result = OperationResult::Failure;

                    journal
                        .borrow_mut()
                        .push(entry)
                        .map_err(|_| ApiWriteConfigError::JournalFull)?;

                    return Err(ApiWriteConfigError::ValidateError(e));
                }

                journal
                    .borrow_mut()
                    .push(entry)
                    .map_err(|_| ApiWriteConfigError::JournalFull)?;

                unsafe {
                    *(bytes as *mut T::Type) = value.clone();
                }

                Ok(())
            },
        );

        // used by the property itself
        let validate_and_record = Box::new(
            move |journal: &mut JournalHandle<MachineConfigMutation>,
                  value: &T::Type|
                  -> Result<(), WriteConfigError> {
                let mut entry = MachineConfigMutation {
                    target: ident,
                    resource_path: Cow::Borrowed(path),
                    value: T::into_string(value),
                    origin: OperationOrigin::Machine,
                    result: OperationResult::Success,
                    timestamp: Utc::now(),
                };

                if let Err(e) = value.validate(options.min, options.max) {
                    entry.result = OperationResult::Failure;
                    journal
                        .append(entry)
                        .map_err(|_| WriteConfigError::JournalFull)?;
                    return Err(WriteConfigError::ValueOutOfBounds(e));
                };

                if let Some(func) = options.validate
                    && let Err(e) = (func)(value)
                {
                    entry.result = OperationResult::Failure;
                    journal
                        .append(entry)
                        .map_err(|_| WriteConfigError::JournalFull)?;
                    return Err(WriteConfigError::ValidateError(e));
                }

                Ok(())
            },
        );

        Ok(self.config_properties.register::<T::Type>(
            self.ident,
            path,
            options.default,
            write_api,
            validate_and_record,
            T::extract,
        )?)
    }
}

// --- state property ---
#[derive(Debug, Default)]
pub struct StatePropertyOptions<T: Default> {
    pub initial_value: T,
}

impl<'a> BuildContext<'a> {
    pub fn register_state_property<T>(
        &mut self,
        path: &'static str,
        options: StatePropertyOptions<T::Type>,
    ) -> Result<StateProperty<T::Type>, BuildError>
    where
        T: TypeWrapper + Extract<ScalarValue> + 'static,
        T::Type: Default,
    {
        Ok(self
            .state_properties
            .register(self.ident, path, options.initial_value, T::extract)?)
    }
}

// --- measurements ---
#[derive(Debug, Default)]
pub struct MeasurementOptions<T> {
    pub initial_value: Option<T>,
    pub record_min: bool,
    pub record_max: bool,
}

impl<'a> BuildContext<'a> {
    pub fn register_measurement<T>(
        &mut self,
        path: &'static str,
        options: MeasurementOptions<T::Type>,
    ) -> Result<Measurement<T::Type>, BuildError>
    where
        T: TypeWrapper + Extract<Option<f64>> + 'static,
        T::Type: Copy + Default,
    {
        Ok(self.measurements.register(
            self.ident,
            path,
            options.initial_value,
            options.record_min,
            options.record_max,
            T::extract,
        )?)
    }
}

// --- command ---
impl<'a> BuildContext<'a> {
    pub fn register_command<M, T>(
        &mut self,
        path: &'static str,
        execute: fn(&mut M, T) -> Result<(), CommandExecuteError>,
    ) -> Result<CommandHandle, BuildError>
    where
        M: Machine + 'static,
        T: serde::de::DeserializeOwned + 'static,
    {
        Ok(self.commands.register(
            self.ident,
            path,
            Box::new(move |machine: &mut dyn Machine, bytes: &str| {
                let machine_type_name = any::type_name_of_val(machine);
                let any: &mut dyn Any = machine;

                let machine =
                    any.downcast_mut::<M>()
                        .ok_or(CommandExecuteError::UnexpectedMachineType {
                            expected: any::type_name::<M>(),
                            received: machine_type_name,
                        })?;

                let args: T = match serde_json::from_str(bytes) {
                    Ok(v) => v,
                    Err(e) => return Err(CommandExecuteError::ParsingError(e)),
                };

                execute(machine, args)
            }),
        )?)
    }
}

// --- event ---
impl<'a> BuildContext<'a> {
    pub fn register_event<T>(&mut self, path: &'static str) -> Result<EventEmitter<T>, BuildError>
    where
        T: Serialize,
    {
        Ok(self.events.create(self.ident, path)?)
    }
}

// --- misc ---
pub trait BoundedMeta {
    type Bound: Copy + PartialOrd + Debug;

    fn validate(
        &self,
        min: Option<Self::Bound>,
        max: Option<Self::Bound>,
    ) -> Result<Self::Bound, BoundsError>;
}
