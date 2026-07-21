use crate::conversion::{Bounded, FloatPropertyType, PropertyType, ScalarPropertyType};
use crate::resource::{ConfigProperty, ConfigPropertyOptions, Measurement, MeasurementOptions, StateProperty, StatePropertyOptions};
use super::{BuildContext, BuildError};

impl<'a> BuildContext<'a> {
    pub fn register_config_property<T>(
        &mut self, 
        name: &'static str, 
        options: ConfigPropertyOptions<T::Value>,
    ) -> Result<ConfigProperty<T>, BuildError> 
    where 
        T: ScalarPropertyType + 'static,
        <T as PropertyType>::Value: Bounded
    {
        Ok(self.config_properties.register::<T>(self.ident, name, options)?)
    }

    pub fn register_state_property<T>(
        &mut self, 
        name: &'static str, 
        options: StatePropertyOptions<T::Value>,
    ) -> Result<StateProperty<T>, BuildError> 
    where 
        T: ScalarPropertyType + 'static,
        <T as PropertyType>::Value: Bounded
    {
        Ok(self.state_properties.register::<T>(self.ident, name, options)?)
    }

    pub fn register_measurement<T>(
        &mut self, 
        name: &'static str, 
        options: MeasurementOptions<T::Value>,
    ) -> Result<Measurement<T::Value>, BuildError> 
    where 
        T: FloatPropertyType + 'static,
        <T as PropertyType>::Value: Bounded
    {
        Ok(self.measurements.register::<T>(self.ident, name, options)?)
    }
}
