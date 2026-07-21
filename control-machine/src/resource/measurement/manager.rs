use control_core::{MachineIdentificationUnique};
use crate::conversion::{Convertible, FloatPropertyType, PropertyType};

use crate::resource::{
    PropertyAccessHandle, PropertyReader, PropertyRegistry, PropertyResolver, RegisterError, Specification, kind, 
};

use super::{Measurement, MeasurementSpecification};

use crate::resource::REGISTRY_ID_MEASUREMENTS as REGISTRY_ID;
const SLOT_SIZE: usize = 32;
const MAX_ITEMS: usize = 512;

pub type Registry = PropertyRegistry<
    REGISTRY_ID, 
    SLOT_SIZE, 
    MAX_ITEMS, 
    kind::StateProperty, 
    Option<f64>
>;

pub type MeasurementResolver<'a> = PropertyResolver<
    'a, 
    REGISTRY_ID, 
    SLOT_SIZE, 
    MAX_ITEMS, 
    kind::StateProperty, 
    Option<f64>
>;

pub type MeasurementReader<'a> = PropertyReader<
    'a, 
    REGISTRY_ID, 
    SLOT_SIZE, 
    MAX_ITEMS, 
    kind::StateProperty, 
    Option<f64>
>;

pub type MeasurementAccessHandle<T> = PropertyAccessHandle<REGISTRY_ID, T>;

#[derive(Debug)]
pub struct MeasurementManager {
    registry: Registry,
    // stat_list: heapless::Vec<usize, MAX_ITEMS>,
}

impl MeasurementManager {
    pub(crate) fn register<Spec>(
        &mut self,
        ident: MachineIdentificationUnique,
        initial_value: <Spec::Type as PropertyType>::Value,
    ) -> Result<Measurement<Spec::Type>, RegisterError> 
    where 
        Spec: MeasurementSpecification + 'static,
        Spec::Type: FloatPropertyType,
        <Spec as Specification>::Type: Convertible<Option<f64>>,
        <<Spec as Specification>::Type as PropertyType>::Value: Copy,
    {
        let handle = self.registry.register::<Spec>(ident)?;
        handle.write(initial_value);

        if Spec::RECORD_MIN {
            // cannot register as properties, need a different mechanism
        }

        if Spec::RECORD_MAX {
            // cannot register as properties, need a different mechanism
        }

        Ok(Measurement {
            handle,
            // stats: Statistics { min, max },
            // value: initial_value,
        })
    }

    pub fn unregister_machine(&mut self, ident: MachineIdentificationUnique) -> usize {
        self.registry.unregister_machine(ident)
    }
}
