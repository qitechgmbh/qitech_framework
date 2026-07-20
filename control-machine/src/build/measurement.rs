use std::marker::PhantomData;

use crate::conversion::Wrapped;
use crate::resource::{Measurement, MeasurementSpec};
use super::{BuildContext, BuildError};

impl<'a> BuildContext<'a> {
    pub fn measurement<'b, T>(&'b mut self) -> MeasurementBuilder<'a, 'b, T>
    where
        'a: 'b,
        T: MeasurementSpec
    {
        MeasurementBuilder { root: self, _marker: PhantomData }
    }
}

pub struct MeasurementBuilder<'a, 'b, T: MeasurementSpec> {
    root: &'b mut BuildContext<'a>,
    _marker: PhantomData<T>,
}

impl<T: MeasurementSpec> MeasurementBuilder<'_, '_, T> 
where 
    <T::Value as Wrapped>::Inner: Default
{
    pub fn register(self) -> Result<Measurement<T::Value>, BuildError> {
        let out = self.root.measurements.register(
            self.root.ident, 
            T::NAME,
            T::initial_value(),
            T::RECORD_MIN,
            T::RECORD_MAX,
        )?;

        Ok(out)
    }
}
