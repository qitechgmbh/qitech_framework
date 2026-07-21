use control_core::{MachineIdentificationUnique};
use crate::conversion::FloatPropertyType;
use crate::resource::{
    kind, 
    RegisterError, 
    PropertyRegistry, 
    PropertyResolver, 
    PropertyReader, 
    PropertyAccessHandle, 
};

use super::{Measurement, MeasurementOptions};

const SLOT_SIZE: usize = 32;
const MAX_ITEMS: usize = 512;

pub type Registry = PropertyRegistry<
    SLOT_SIZE, 
    MAX_ITEMS, 
    kind::StateProperty,
    Option<f64>
>;

pub type MeasurementResolver<'a> = PropertyResolver<
    'a, 
    SLOT_SIZE, 
    MAX_ITEMS, 
    kind::StateProperty, 
    Option<f64>
>;

pub type MeasurementReader<'a> = PropertyReader<
    'a, 
    SLOT_SIZE, 
    MAX_ITEMS, 
    kind::StateProperty, 
    Option<f64>
>;

pub type MeasurementAccessHandle<T> = PropertyAccessHandle<kind::StateProperty, T>;

#[derive(Debug)]
pub struct MeasurementManager {
    registry: Registry,
}

impl MeasurementManager {
    pub(crate) fn register<T>(
        &mut self,
        ident: MachineIdentificationUnique,
        name: &'static str,
        options: MeasurementOptions<T::Value>,
    ) -> Result<Measurement<T::Value>, RegisterError> 
    where 
        T: FloatPropertyType + 'static
    {
        let handle = self.registry.register::<T>(ident, name, "")?;
        handle.write(options.initial_value.unwrap_or_default());

        if options.record_min {
            // TODO: implement
        }

        if options.record_max {
            // TODO: implement
        }

        Ok(Measurement { handle })
    }

    pub fn unregister_machine(&mut self, ident: MachineIdentificationUnique) -> usize {
        self.registry.unregister_machine(ident)
    }
}
