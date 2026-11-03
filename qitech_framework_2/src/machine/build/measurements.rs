use std::any::type_name;
use std::borrow::Cow;

use qitech_framework_core::report::ResourceKind;
use qitech_framework_core::report::error::BuildError;
use qitech_framework_core::schema::MeasurementKind;

use crate::machine::BuildContext;
use crate::machine::measurement::Measurement;
use crate::machine::measurement::MeasurementStatistics;
use crate::resource::conversion::PropertyAdapter;
use crate::resource::conversion::ReadMeasurement;
use crate::resource::conversion::StatisticValue;

impl<'a> BuildContext<'a> {
    pub fn measurement<'b, T>(&'b mut self, path: &'static str) -> MeasurementBuilder<'a, 'b, T>
    where
        'a: 'b,
        T: PropertyAdapter + 'static,
        T::Type: Copy + StatisticValue,
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
    value: T::Type,
}

impl<'a, 'b, T> MeasurementBuilder<'a, 'b, T>
where
    T: PropertyAdapter + ReadMeasurement + 'static,
    T::Type: Copy + StatisticValue,
{
    pub fn initial(mut self, value: T::Input) -> Self {
        self.value = T::convert_input(value);
        self
    }

    pub fn build(self) -> Result<Measurement<T::Type>, BuildError> {
        let Some(def) = self.root.schema.measurements.get(self.path) else {
            return Err(BuildError::IllegalResourcePath {
                kind: ResourceKind::Measurement,
                path: self.path.to_string(),
            });
        };

        if !T::validate_measurement_definition(def, false) {
            return Err(BuildError::IllegalResourceType {
                kind: ResourceKind::Measurement,
                path: self.path.to_string(),
                expected: format!("{}", def.kind),
                received: type_name::<T>().to_string(),
            });
        }

        if !self.root.measurements_registered.insert(self.path) {
            return Err(BuildError::DuplicateResource(self.path.to_string()));
        }

        let p_value = self.root.measurements.register::<T::Type>(
            self.root.ident,
            Cow::Borrowed(self.path),
            self.value,
            T::read,
        );

        // --- create statistics ---
        let mut register = |suffix: &str| {
            self.root.measurements.register::<T::Type>(
                self.root.ident,
                Cow::Owned(format!("{}.{}", self.path, suffix)),
                self.value,
                T::read,
            )
        };

        let (min, max, avg, stddev) = match &def.kind {
            MeasurementKind::Boolean => (None, None, None, None),

            MeasurementKind::Integer { statistics } | MeasurementKind::Float { statistics, .. } => {
                (
                    statistics.min.then(|| register("min")),
                    statistics.max.then(|| register("max")),
                    statistics.avg.then(|| register("avg")),
                    statistics.stddev.then(|| register("stddev")),
                )
            }
        };

        let stats = MeasurementStatistics {
            p_generation: self.root.export_count.clone(),
            generation: 0,
            min,
            max,
            avg,
            stddev,
            count: 0,
            mean: 0.0,
            m2: 0.0,
        };

        Ok(Measurement { p_value, stats })
    }
}
