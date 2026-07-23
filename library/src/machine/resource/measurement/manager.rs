use qitech_framework_common::MachineIdentificationUnique;

use crate::machine::resource::{
    PropertyAccessHandle, PropertyReader, PropertyRegistry, 
    PropertyResolver, error::RegisterResult, kind, property::Extract,
};
use super::Measurement;

const SLOT_SIZE: usize = size_of::<f64>();
const MAX_ITEMS: usize = 512;

pub type Registry = PropertyRegistry<
    SLOT_SIZE, 
    MAX_ITEMS, 
    kind::StateProperty,
    Option<f64>,
>;

pub type Resolver<'a> = PropertyResolver<
    'a, 
    SLOT_SIZE, 
    MAX_ITEMS, 
    kind::StateProperty, 
    Option<f64>,
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
pub struct Manager {
    registry: Registry,
}

impl Manager {
    pub(crate) fn register<T: Default + 'static>(
        &mut self,
        ident: MachineIdentificationUnique,
        name: &'static str,
        initial_value: Option<T>,
        record_min: bool,
        record_max: bool,
        extract: Extract<Option<f64>>,
    ) -> RegisterResult<Measurement<T>> {
        let handle = self.registry.register::<T>(ident, name, "", extract, ())?;
        handle.write(initial_value.unwrap_or_default());

        if record_min {
            // TODO: implement
        }

        if record_max {
            // TODO: implement
        }

        Ok(Measurement { handle })
    }

    pub fn unregister_machine(&mut self, ident: MachineIdentificationUnique) -> usize {
        self.registry.unregister_machine(ident)
    }
}
