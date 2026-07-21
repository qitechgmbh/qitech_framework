use crate::conversion::{FloatPropertyType, ScalarPropertyType};
use crate::resource::{ConfigProperty, ConfigPropertyOptions, Measurement, MeasurementOptions, StateProperty, StatePropertyOptions};
use super::{BuildContext, BuildError};

impl<'a> BuildContext<'a> {
    pub fn register_config_property<T: ScalarPropertyType>(
        &mut self, 
        name: &'static str, 
        options: ConfigPropertyOptions<T::Value>,
    ) -> Result<ConfigProperty<T>, BuildError> {
        self.config_properties.register::<T>(self.ident, name, options).into()
    }

    pub fn register_state<T: ScalarPropertyType>(
        &mut self, 
        name: &'static str, 
        options: StatePropertyOptions<T::Value>,
    ) -> Result<StateProperty<T>, BuildError> {
        self.state_properties.register::<T>(self.ident, name, options).into()
    }

    pub fn register_measurement<T: FloatPropertyType>(
        &mut self, 
        name: &'static str, 
        options: MeasurementOptions<T::Value>,
    ) -> Result<Measurement<T>, BuildError> {
        self.measurements.register::<T>(self.ident, name, options).into()
    }
}
