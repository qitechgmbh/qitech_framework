use std::marker::PhantomData;

use crate::conversion::{Convertible, FloatPropertyType, PropertyType};
use crate::resource::{Measurement, MeasurementSpecification, Specification};
use super::{BuildContext, BuildError};

pub struct MeasurementOptions {
    record_min: bool,
    record_max: bool,
}

impl<'a> BuildContext<'a> {
    pub fn register_measurement<'b, T>(
        &'b mut self, 
        name: &'static str, 
        options: MeasurementOptions
    ) -> Result<Measurement<T::Value>, BuildError>
    where
        'a: 'b,
        T: FloatPropertyType
    {
        let out = self.measurements.register::<T>(
            self.ident, 
            options,
        )
    }
}

pub struct MeasurementBuilder<'a, 'b, T>
where 
    T: FloatPropertyType
{
    root: &'b mut BuildContext<'a>,
    _marker: PhantomData<T>,
}

impl<T> MeasurementBuilder<'_, '_, T> 
where 
    T: FloatPropertyType
{
    pub fn register(self) -> Result<Measurement<T::Type>, BuildError> {
        
        let out = self.root.measurements.register::<T>(
            self.root.ident, 
            T::initial_value(),
        )?;

        Ok(out)
    }
}
