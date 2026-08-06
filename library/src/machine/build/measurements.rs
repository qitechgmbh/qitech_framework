use chrono::Utc;
use qitech_framework_core::report::StatePropertyWriteRecord;

use crate::machine::BuildContext;
use crate::machine::build::BuildResult;
use crate::resource::Measurement;
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
    T: PropertyAdapter + 'static,
    T::Type: Copy,
{
    pub fn initial(mut self, value: T::Input) -> Self {
        self.value = T::convert_input(value);
        self
    }

    pub fn register(self) -> BuildResult<Measurement<T::Type>> {
        // TODO: catch register error
        let handle = self
            .root
            .measurements
            .register::<T::Type>(self.path, self.value.clone());

        let timestamp = Utc::now();

        // TODO: expose a temp journal so on failure we don't send this out
        self.root
            .journals
            .state_property_write
            .new_handle()
            .append(StatePropertyWriteRecord {
                ident: self.root.ident,
                path: self.path.to_string(),
                value: T::into_scalar(self.value.clone()),
                timestamp,
            });

        let prop = Measurement::new(handle);
        Ok(prop)
    }
}
