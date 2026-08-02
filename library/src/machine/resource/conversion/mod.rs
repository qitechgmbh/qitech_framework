use qitech_framework_core::ScalarValue;
use qitech_framework_core::with_uom_quantities;
use qitech_framework_core::with_uom_units;

mod type_wrapper;
pub use type_wrapper::TypeWrapper;

pub trait Extract<T> {
    /// Extracts a value from a raw byte pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` is valid for reads of the required
    /// size and properly aligned for the expected type.
    unsafe fn extract(bytes: *const u8) -> T;
}

/// trait each measurement types value type must implement
pub trait StatisticValue: Copy + Default + PartialOrd {
    fn as_opt_f64(self) -> Option<f64>;
    fn from_f64(value: f64) -> Self;
    fn zero() -> Self;
}

impl StatisticValue for f64 {
    fn as_opt_f64(self) -> Option<f64> {
        Some(self)
    }

    fn from_f64(value: f64) -> Self {
        value
    }

    fn zero() -> Self {
        0.0
    }
}

// --- float ---
impl Extract<Option<f64>> for f64 {
    unsafe fn extract(bytes: *const u8) -> Option<f64> {
        let value = unsafe { *(bytes as *const f64) };
        Some(value)
    }
}

impl Extract<ScalarValue> for f64 {
    unsafe fn extract(bytes: *const u8) -> ScalarValue {
        let value = unsafe { *(bytes as *const f64) };
        ScalarValue::Float(Some(value))
    }
}

// --- optional float ---
impl Extract<Option<f64>> for Option<f64> {
    unsafe fn extract(bytes: *const u8) -> Option<f64> {
        unsafe { *(bytes as *const Option<f64>) }
    }
}

impl Extract<ScalarValue> for Option<f64> {
    unsafe fn extract(bytes: *const u8) -> ScalarValue {
        let value = unsafe { *(bytes as *const Option<f64>) };
        ScalarValue::Float(value)
    }
}

// --- integer ---
impl Extract<Option<f64>> for i64 {
    unsafe fn extract(bytes: *const u8) -> Option<f64> {
        let value = unsafe { *(bytes as *const i64) };
        Some(value as f64)
    }
}

impl Extract<ScalarValue> for i64 {
    unsafe fn extract(bytes: *const u8) -> ScalarValue {
        let value = unsafe { *(bytes as *const i64) };
        ScalarValue::Integer(Some(value))
    }
}

// --- optional int ---
impl Extract<Option<f64>> for Option<i64> {
    unsafe fn extract(bytes: *const u8) -> Option<f64> {
        let value = unsafe { *(bytes as *const Option<i64>) };
        value.map(|v| v as f64)
    }
}

impl Extract<ScalarValue> for Option<i64> {
    unsafe fn extract(bytes: *const u8) -> ScalarValue {
        let value = unsafe { *(bytes as *const Option<i64>) };
        ScalarValue::Integer(value)
    }
}

// --- bool ---
impl Extract<Option<f64>> for bool {
    unsafe fn extract(bytes: *const u8) -> Option<f64> {
        let value = unsafe { *(bytes as *const bool) };
        Some(if value { 1.0 } else { 0.0 })
    }
}

impl Extract<ScalarValue> for bool {
    unsafe fn extract(bytes: *const u8) -> ScalarValue {
        let value = unsafe { *(bytes as *const bool) };
        ScalarValue::Boolean(Some(value))
    }
}

// --- optional bool ---
impl Extract<Option<f64>> for Option<bool> {
    unsafe fn extract(bytes: *const u8) -> Option<f64> {
        let value = unsafe { *(bytes as *const Option<bool>) };
        value.map(|b| if b { 1.0 } else { 0.0 })
    }
}

impl Extract<ScalarValue> for Option<bool> {
    unsafe fn extract(bytes: *const u8) -> ScalarValue {
        let value = unsafe { *(bytes as *const Option<bool>) };
        ScalarValue::Boolean(value)
    }
}

// --- string ---
impl Extract<ScalarValue> for String {
    unsafe fn extract(bytes: *const u8) -> ScalarValue {
        let value = unsafe { (*(bytes as *const String)).clone() };
        ScalarValue::String(Some(value.clone()))
    }
}

// impl Extract<ScalarValue> for Option<String> {
//     unsafe fn extract(bytes: *const u8) -> ScalarValue {
//         let value = unsafe { (*(bytes as *const Option<String>)).clone() };
//         ScalarValue::String(value.clone().map(Cow::Owned))
//     }
// }

// --- uom quantities ---
macro_rules! impl_uom_unit {
    ($quantity:path, $unit:path, $unit_trait:path, $conversion_trait:path) => {
        impl TypeWrapper for $unit {
            type Type = $quantity;
            type Input = f64;

            fn convert_input(input: f64) -> $quantity {
                <$quantity>::new::<$unit>(input)
            }

            fn from_scalar(value: ScalarValue) -> Option<Self::Type> {
                match value {
                    ScalarValue::Float(Some(x)) => Some(<$quantity>::new::<$unit>(x)),
                    _ => None,
                }
            }

            fn into_scalar(value: &Self::Type) -> ScalarValue {
                ScalarValue::Float(Some(value.get::<$unit>()))
            }
        }

        impl Extract<Option<f64>> for $unit {
            unsafe fn extract(bytes: *const u8) -> Option<f64> {
                let value = unsafe { *(bytes as *const $quantity) };
                Some(value.get::<$unit>())
            }
        }

        impl Extract<ScalarValue> for $unit {
            unsafe fn extract(bytes: *const u8) -> ScalarValue {
                let value = unsafe { *(bytes as *const $quantity) };
                ScalarValue::Float(Some(value.get::<$unit>()))
            }
        }

        // --- optional ---
        impl TypeWrapper for Option<$unit> {
            type Type = Option<$quantity>;
            type Input = Option<f64>;

            fn convert_input(input: Option<f64>) -> Option<$quantity> {
                input.map(|x| <$quantity>::new::<$unit>(x))
            }

            fn from_scalar(value: ScalarValue) -> Option<Self::Type> {
                match value {
                    ScalarValue::Float(v) => Some(v.map(|x| <$quantity>::new::<$unit>(x))),
                    _ => None,
                }
            }

            fn into_scalar(value: &Self::Type) -> ScalarValue {
                ScalarValue::Float(value.map(|x| x.get::<$unit>()))
            }
        }

        impl Extract<Option<f64>> for Option<$unit> {
            unsafe fn extract(bytes: *const u8) -> Option<f64> {
                let value = unsafe { *(bytes as *const Option<$quantity>) };
                value.map(|x| x.get::<$unit>())
            }
        }

        impl Extract<ScalarValue> for Option<$unit> {
            unsafe fn extract(bytes: *const u8) -> ScalarValue {
                let value = unsafe { *(bytes as *const Option<$quantity>) };
                ScalarValue::Float(value.map(|x| x.get::<$unit>()))
            }
        }
    };
}

with_uom_units!(impl_uom_unit);

// --- quantity ---
macro_rules! impl_uom_quantity {
    ($quantity:path, $unit_trait:path, $conversion_trait:path) => {
        impl BoundedMeta for $quantity {
            type Bound = $quantity;

            fn as_bound(&self) -> Option<$quantity> {
                Some(*self)
            }
        }
    };
}

with_uom_quantities!(impl_uom_quantity);
