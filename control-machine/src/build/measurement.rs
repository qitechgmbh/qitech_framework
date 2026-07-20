use crate::{build::{MachineBuildContext, MachineBuildError}, conversion::Wrapped, resource::{Measurement, RegisterError}};

impl<'a> MachineBuildContext<'a> {
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
    root: &'b mut MachineBuildContext<'a>,
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
    pub fn with_record_min(&mut self) -> &mut Self {
        self.record_min = true;
        self
    }

    pub fn with_record_max(&mut self) -> &mut Self {
        self.record_max = true;
        self
    }

    pub fn with_initial_value(&mut self, value: T::Inner) -> &mut Self {
        self.initial_value = value;
        self
    }

    pub fn register(self) -> Result<Measurement<T>, MachineBuildError> {
        let out = self.root.measurements.register(
            self.root.ident, 
            self.name, 
            ,
            initial_value
        );
    }
}

impl From<RegisterError> for MachineBuildError {
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
