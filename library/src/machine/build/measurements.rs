use qitech_framework_core::report::error::BuildError;

use crate::machine::BuildContext;
use crate::machine::conversion::Extract;
use crate::machine::conversion::PropertyAdapter;
use crate::machine::measurement::Measurement;

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
        if !self.root.measurements_registered.insert(self.path) {
            return Err(BuildError::DuplicateResource(self.path.to_string()));
        }

        let p_value =
            self.root
                .measurements
                .register::<T::Type>(self.root.ident, self.path, self.value);

        Ok(Measurement { p_value })
    }
}
