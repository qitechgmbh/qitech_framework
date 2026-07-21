use crate::{MachineBuildError, MachineBuildContext, conversion::Property, resource};
use super::super::{Measurement, MeasurementStatistics};

impl<'a> MachineBuildContext<'a> {
    pub fn measurement<'b, T>(
        &'b mut self,
        name: &'static str,
    ) -> MeasurementBuilder<'a, 'b, T>
    where
        'a: 'b,
        T: Property,
        T::Inner: Default
    {
        MeasurementBuilder {
            root: self,
            name,
            record_min: false,
            record_max: false,
            initial_value: Default::default(),
        }
    }
}

pub struct MeasurementBuilder<'a, 'b, T>
where
    T: Property
{
    root: &'b mut MachineBuildContext<'a>,
    name: &'static str,
    record_min: bool,
    record_max: bool,
    initial_value: T::Inner,
}

impl<T> MeasurementBuilder<'_, '_, T>
where
    T: Property + 'static,
    T::Inner: Default + Copy
{
    pub fn record_min(&mut self) -> &mut Self {
        self.record_min = true;
        self
    }

    pub fn record_max(&mut self) -> &mut Self {
        self.record_max = true;
        self
    }

    pub fn initial_value(&mut self, value: T::Inner) -> &mut Self {
        self.initial_value = value;
        self
    }

    pub fn register(self) -> Result<Measurement<T>, MachineBuildError> {
        let ident = self.root.ident;

        let name = self.root.register_name(self.name);

        let reg = &mut self.root.resource_registry;
        let handle = reg.register_measurement::<T>(ident, name)?;

        let min = if self.record_min {
            let name = reg.register_name(format!("{name}.min"));
            let handle = reg.register_measurement::<T>(ident, name)?;
            Some(handle)
        } else {
            None
        };

        let max = if self.record_max {
            let name = reg.register_name(format!("{name}.max"));
            let handle = reg.register_measurement::<T>(ident, name)?;
            Some(handle)
        } else {
            None
        };

        let stats = MeasurementStatistics::new(min, max);
        Ok(Measurement::new(handle, stats, self.initial_value))
    }
}

impl From<resource::measurement::RegisterError> for MachineBuildError {
    fn from(value: resource::measurement::RegisterError) -> Self {
        use resource::measurement::RegisterError::*;
        match value {
            AlreadyRegistered { name } => MachineBuildError::AlreadyRegistered { 
                registry: "measurements",  
                name 
            },
            RegistryFull { name }  => MachineBuildError::RegistryFull { 
                registry: "measurements",  
                name,  
            },
        }
    }
}