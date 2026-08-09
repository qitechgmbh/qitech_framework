use qitech_framework_core::with_uom_quantities;

pub trait StatisticValue: Copy + Default + PartialOrd {
    fn as_opt_f64(self) -> Option<f64>;
    fn from_f64(value: f64) -> Self;
}

impl StatisticValue for f64 {
    fn as_opt_f64(self) -> Option<f64> {
        Some(self)
    }

    fn from_f64(value: f64) -> Self {
        value
    }
}

impl StatisticValue for i64 {
    fn as_opt_f64(self) -> Option<f64> {
        Some(self as f64)
    }

    fn from_f64(value: f64) -> Self {
        value.round() as i64
    }
}

impl StatisticValue for bool {
    fn as_opt_f64(self) -> Option<f64> {
        // Bools have no meaningful numeric mean/stddev — opt them out
        // of avg/stddev accumulation entirely (min/max still works via
        // PartialOrd: false < true).
        None
    }

    fn from_f64(value: f64) -> Self {
        value != 0.0
    }
}

impl<T: StatisticValue> StatisticValue for Option<T> {
    fn as_opt_f64(self) -> Option<f64> {
        self.and_then(T::as_opt_f64)
    }

    fn from_f64(value: f64) -> Self {
        Some(T::from_f64(value))
    }
}

// --- quantity ---
macro_rules! impl_uom_quantity {
    ($quantity:path, $unit:path, $conversion_trait:path) => {
        impl StatisticValue for $quantity {
            fn as_opt_f64(self) -> Option<f64> {
                Some(self.value)
            }

            fn from_f64(value: f64) -> Self {
                Self {
                    dimension: core::marker::PhantomData,
                    units: core::marker::PhantomData,
                    value,
                }
            }
        }
    };
}

with_uom_quantities!(impl_uom_quantity);
