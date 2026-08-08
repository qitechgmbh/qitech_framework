use std::borrow::Cow;

use qitech_framework_core::report::error::BuildError;

use crate::machine::BuildContext;
use crate::machine::conversion::Extract;
use crate::machine::conversion::PropertyAdapter;
use crate::machine::conversion::StatisticValue;
use crate::machine::measurement::Measurement;
use crate::machine::measurement::MeasurementStatistics;

impl<'a> BuildContext<'a> {
    pub fn measurement<'b, T>(&'b mut self, path: &'static str) -> MeasurementBuilder<'a, 'b, T>
    where
        'a: 'b,
        T: PropertyAdapter + StatisticValue + 'static,
        T::Type: Copy + StatisticValue,
    {
        MeasurementBuilder {
            root: self,
            path,
            value: T::Type::default(),
            record_min: false,
            record_max: false,
            record_avg: false,
            record_stddev: false,
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

    record_min: bool,
    record_max: bool,
    record_avg: bool,
    record_stddev: bool,
}

impl<'a, 'b, T> MeasurementBuilder<'a, 'b, T>
where
    T: PropertyAdapter + Extract<Option<f64>> + 'static,
    T::Type: Copy + StatisticValue,
{
    pub fn initial(mut self, value: T::Input) -> Self {
        self.value = T::convert_input(value);
        self
    }

    pub fn record_min(mut self) -> Self {
        self.record_min = true;
        self
    }

    pub fn record_max(mut self) -> Self {
        self.record_max = true;
        self
    }

    pub fn record_avg(mut self) -> Self {
        self.record_avg = true;
        self
    }

    pub fn record_stddev(mut self) -> Self {
        self.record_stddev = true;
        self
    }

    pub fn register(self) -> Result<Measurement<T::Type>, BuildError> {
        if !self.root.measurements_registered.insert(self.path) {
            return Err(BuildError::DuplicateResource(self.path.to_string()));
        }

        let p_value = self.root.measurements.register::<T::Type>(
            self.root.ident,
            Cow::Borrowed(self.path),
            self.value,
            T::extract,
        );

        let min = if self.record_min {
            Some(self.root.measurements.register::<T::Type>(
                self.root.ident,
                Cow::Owned(format!("{}.{}", self.path, "min")),
                self.value,
                T::extract,
            ))
        } else {
            None
        };

        let max = if self.record_max {
            Some(self.root.measurements.register::<T::Type>(
                self.root.ident,
                Cow::Owned(format!("{}.{}", self.path, "max")),
                self.value,
                T::extract,
            ))
        } else {
            None
        };

        let avg = if self.record_avg {
            Some(self.root.measurements.register::<T::Type>(
                self.root.ident,
                Cow::Owned(format!("{}.{}", self.path, "avg")),
                self.value,
                T::extract,
            ))
        } else {
            None
        };

        let stddev = if self.record_stddev {
            Some(self.root.measurements.register::<T::Type>(
                self.root.ident,
                Cow::Owned(format!("{}.{}", self.path, "stddev")),
                self.value,
                T::extract,
            ))
        } else {
            None
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

        Ok(Measurement { 
            p_value,
            stats
        })
    }
}
