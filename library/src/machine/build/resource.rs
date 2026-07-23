use std::{any::{self, Any}, default, fmt::Debug};

use qitech_framework_common::ScalarValue;
use serde::{Serialize, de::DeserializeOwned};

use crate::machine::{Machine, build::conversion::{Extract, ScalarTypeWrapper, TypeWrapper}, error::{BoundsError, ValidateError}, resource::{
    command::{CommandError, CommandHandle}, config_property::ConfigProperty, event::EventEmitter, measurement::{Measurement, MeasurementOptions}, state_property::{StateProperty, StatePropertyOptions}, 
}};

use super::{BuildContext, error::BuildError};
// --- config property ---
#[derive(Debug, Default)]
pub struct ConfigPropertyOptions<T: BoundedMeta> {
    default: T,
    min: Option<T::Bound>,
    max: Option<T::Bound>,
    validate: Option<fn(&T) -> Result<(), String>>,
}

impl<'a> BuildContext<'a> {
    pub fn create_config_property<T>(
        &mut self, 
        path: &'static str, 
        options: ConfigPropertyOptions<T::Type>,
    ) -> Result<ConfigProperty<T::Type>, BuildError> 
    where 
        T: ScalarTypeWrapper,
        T::Type: BoundedMeta + Clone,
    {
        let api_validate = {
            let min = options.min;
            let max = options.max;
            let custom = options.validate;

            Some(Box::new(move |value: &T::Type| -> Result<(), ValidateError> {
                if let Err(e) = value.validate(min, max) {
                    return Err(ValidateError::OutOfBounds(e));
                }

                if let Some(validate) = &custom && let Err(e) = validate(value) {
                    return Err(ValidateError::Custom(e));
                }

                Ok(())
            }) as Box<dyn Fn(&T::Type) -> Result<(), ValidateError>>)
        };

        Ok(self.config_properties.create::<T::Type>(
            self.ident, 
            path, 
            options.default,
            api_validate,
            T::extract
        )?)
    }
}

// --- state property ---
impl<'a> BuildContext<'a> {
    pub fn register_state_property<T>(
        &mut self, 
        path: &'static str, 
        options: StatePropertyOptions<T::Type>,
    ) -> Result<StateProperty<T::Type>, BuildError> 
    where 
        T: TypeWrapper + Extract<ScalarValue> + 'static,
        T::Type: Default
    {
        Ok(self.state_properties.register(self.ident, path, options, T::extract)?)
    }
}


// --- measurements ---
pub struct MeasurementOptions<T> {
    initial_value: Option<T>,
    record_min: bool,
    record_max: bool,
}

impl<'a> BuildContext<'a> {
    pub fn register_measurement<T>(
        &mut self, 
        path: &'static str, 
        options: MeasurementOptions<T::Type>,
    ) -> Result<Measurement<T::Type>, BuildError> 
    where 
        T: TypeWrapper + Extract<Option<f64>> + 'static,
        T::Type: Copy + Default
    {
        Ok(self.measurements.register(
            self.ident, 
            path, 
            options.initial_value,
            options.record_min,
            options.record_max,
            T::extract
        )?)
    }
}

// --- command ---
impl<'a> BuildContext<'a> {
    pub fn register_command<M, T>(
        &mut self,
        path: &'static str,
        execute: fn(&mut M, T) -> Result<(), CommandError>,
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

                let machine = any
                    .downcast_mut::<M>()
                    .ok_or(CommandError::UnexpectedMachineType { 
                        expected: any::type_name::<M>(),
                        received: machine_type_name,
                    })?;

                let args: T = match serde_json::from_str(bytes) {
                    Ok(v) => v,
                    Err(e) => return Err(CommandError::ParsingError(e)),
                };

                execute(machine, args)
            }),
        )?)
    }
}

// --- event ---
impl<'a> BuildContext<'a> {
    pub fn register_event<T>(
        &mut self, 
        path: &'static str, 
    ) -> Result<EventEmitter<T>, BuildError> 
    where 
        T: Serialize,
    {
        Ok(self.events.create(self.ident, path)?)
    }
}

// --- misc ---
trait BoundedMeta { 
    type Bound: Copy + PartialOrd + Debug;

    fn validate(
        &self,
        min: Option<Self::Bound>,
        max: Option<Self::Bound>,
    ) -> Result<Self::Bound, BoundsError>;
}
