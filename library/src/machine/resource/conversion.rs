use std::fmt::Debug;

use qitech_framework_core::ScalarValue;
use qitech_framework_core::with_uom_quantities;
use qitech_framework_core::with_uom_units;

pub trait Extract<T> {
    /// Extracts a value from a raw byte pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` is valid for reads of the required
    /// size and properly aligned for the expected type.
    unsafe fn extract(bytes: *const u8) -> T;
}

pub trait BoundedMeta {
    type Bound: Copy + Default + PartialOrd + Debug;
    fn as_bound(&self) -> Option<Self::Bound>;
}

/// Trait to allow defining conversion and extract operations
/// on wrapped units. The best example here is uom which allows us to export
/// a uom Length as millimeter instead of meter (the default with serde feature)
pub trait TypeWrapper {
    type Type: Clone + 'static;
    type Input;

    fn into_scalar(value: &Self::Type) -> ScalarValue;
    fn convert_input(input: Self::Input) -> Self::Type;
    fn deserialize_json(raw: &str) -> serde_json::Result<Self::Type>;
}

// --- type wrapper ---q
macro_rules! simple_type_wrapper {
    ($type:ty, $scalar_name:tt) => {
        impl TypeWrapper for $type {
            type Type = $type;
            type Input = $type;

            fn convert_input(input: Self::Input) -> Self::Type {
                input
            }

            fn deserialize_json(raw: &str) -> serde_json::Result<Self::Type> {
                serde_json::from_str(raw)
            }

            fn into_scalar(value: &Self::Type) -> ScalarValue {
                ScalarValue::$scalar_name(Some(*value))
            }
        }

        impl TypeWrapper for Option<$type> {
            type Type = Option<$type>;
            type Input = Option<$type>;

            fn convert_input(input: Self::Input) -> Self::Type {
                input
            }

            fn deserialize_json(raw: &str) -> serde_json::Result<Self::Type> {
                serde_json::from_str(raw)
            }

            fn into_scalar(value: &Self::Type) -> ScalarValue {
                ScalarValue::$scalar_name(value.as_ref().map(|v| v.clone()))
            }
        }
    };
}

simple_type_wrapper!(f64, Float);
simple_type_wrapper!(i64, Integer);
simple_type_wrapper!(bool, Boolean);

impl TypeWrapper for String {
    type Type = String;
    type Input = &'static str;

    fn convert_input(input: &'static str) -> String {
        input.to_string()
    }

    fn deserialize_json(raw: &str) -> serde_json::Result<Self::Type> {
        serde_json::from_str(raw)
    }

    fn into_scalar(value: &Self::Type) -> ScalarValue {
        ScalarValue::String(Some(value.clone()))
    }
}

impl TypeWrapper for Option<String> {
    type Type = Option<String>;
    type Input = Option<&'static str>;

    fn convert_input(input: Option<&'static str>) -> Option<String> {
        input.map(|x| x.to_string())
    }

    fn deserialize_json(raw: &str) -> serde_json::Result<Self::Type> {
        serde_json::from_str(raw)
    }

    fn into_scalar(value: &Self::Type) -> ScalarValue {
        ScalarValue::String(value.clone())
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
impl BoundedMeta for i64 {
    type Bound = i64;

    fn as_bound(&self) -> Option<Self::Bound> {
        Some(*self)
    }
}

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

impl BoundedMeta for String {
    type Bound = usize;

    fn as_bound(&self) -> Option<Self::Bound> {
        Some(self.len())
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

            fn deserialize_json(raw: &str) -> serde_json::Result<Self::Type> {
                let value = serde_json::from_str::<f64>(raw)?;
                Ok(<$quantity>::new::<$unit>(value))
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

            fn deserialize_json(raw: &str) -> serde_json::Result<Self::Type> {
                let value = serde_json::from_str::<Option<f64>>(raw)?;
                Ok(value.map(|x| <$quantity>::new::<$unit>(x)))
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
