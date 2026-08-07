use qitech_framework_core::report::error::BuildError;

use crate::machine::BuildContext;
use crate::resource::Measurement;
use crate::resource::conversion::Extract;
use crate::resource::conversion::PropertyAdapter;

impl<'a> BuildContext<'a> {
    pub fn measurement<'b, T>(&'b mut self, path: &'static str) -> MeasurementBuilder<'a, 'b, T>
    where
        'a: 'b,
        T: PropertyAdapter + 'static,
    {
        MeasurementBuilder {
            root: self,
            path,
            value: T::Type::default(),
        }
    }
}

pub struct MeasurementBuilder<'a, 'b, T>
where
    T: PropertyAdapter + 'static,
{
    root: &'b mut BuildContext<'a>,
    path: &'static str,

    // --- configuration ---
    value: T::Type,
}

impl<'a, 'b, T> MeasurementBuilder<'a, 'b, T>
where
    T: PropertyAdapter + Extract<Option<f64>> + 'static,
    T::Type: Copy,
{
    pub fn initial(mut self, value: T::Input) -> Self {
        self.value = T::convert_input(value);
        self
    }

    pub fn register(self) -> Result<Measurement<T::Type>, BuildError> {
        // TODO: catch register error
        let handle = self
            .root
            .measurements
            .register::<T::Type>(self.path, self.value, T::extract);

        let prop = Measurement::new(handle);
        Ok(prop)
    }
}
