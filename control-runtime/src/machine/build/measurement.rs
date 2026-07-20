use crate::{MachineBuildError, BuildContext, conversion::Wrapped, data};
use super::super::{Measurement, MeasurementStatistics};

impl<'a> BuildContext<'a> {
    pub fn measurement<'b, T>(
        &'b mut self,
        name: &'static str,
    ) -> MeasurementBuilder<'a, 'b, T>
    where
        'a: 'b,
        T: Wrapped,
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
    T: Wrapped
{
    root: &'b mut BuildContext<'a>,
    name: &'static str,
    record_min: bool,
    record_max: bool,
    initial_value: T::Inner,
}

impl<T> MeasurementBuilder<'_, '_, T>
where
    T: Wrapped + 'static,
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

        let handle = self.root.data_store.registry.measurement.register::<T>(ident, name)?;

        let min = if self.record_min {
            let name = self.root.register_name(format!("{name}.min"));
            let handle = self.root.data_store.registry.measurement.register::<T>(ident, name)?;
            Some(handle)
        } else {
            None
        };

        let max = if self.record_max {
            let name = self.root.register_name(format!("{name}.max"));
            let handle = self.root.data_store.registry.measurement.register::<T>(ident, name)?;
            Some(handle)
        } else {
            None
        };

        let stats = MeasurementStatistics::new(min, max);
        Ok(Measurement::new(handle, stats, self.initial_value))
    }
}

impl From<data::measurement::RegisterError> for MachineBuildError {
    fn from(value: data::measurement::RegisterError) -> Self {
        use data::measurement::RegisterError::*;
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