use std::fmt::Debug;

use qitech_framework_common::ScalarValue;
use qitech_framework_common::with_uom_units;

use crate::machine::error::BoundsError;
use crate::uom;

pub type ExtractFn<T> = unsafe fn(*const u8) -> T;

pub trait Extract<T> {
    /// Extracts a value from a raw byte pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` is valid for reads of the required
    /// size and properly aligned for the expected type.
    unsafe fn extract(bytes: *const u8) -> T;
}

/// Trait to allow defining conversion and extract operations
/// on wrapped units. The best example here is uom which allows us to export
/// a uom Length as millimeter instead of meter (the default with serde feature)
pub trait TypeWrapper {
    type Type: Clone + 'static;
}

/// Specialized TypeWrapper for dealing with scalar values.
/// Used by ConfigPropertyManager and StatePropertyManager for registration.
pub trait ScalarTypeWrapper: TypeWrapper + Clone + Extract<ScalarValue> + 'static {
    fn into_scalar(value: &Self::Type) -> ScalarValue;
}

pub trait BoundedMeta {
    type Bound: Copy + PartialOrd + Debug;

    fn validate(
        &self,
        min: Option<Self::Bound>,
        max: Option<Self::Bound>,
    ) -> Result<Self::Bound, BoundsError>;
}

// --- float ---
impl TypeWrapper for f64 {
    type Type = f64;
}

impl ScalarTypeWrapper for f64 {
    fn into_scalar(value: &Self::Type) -> ScalarValue {
        ScalarValue::Float(Some(*value))
    }
}

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
impl TypeWrapper for Option<f64> {
    type Type = Option<f64>;
}

impl ScalarTypeWrapper for Option<f64> {
    fn into_scalar(value: &Self::Type) -> ScalarValue {
        ScalarValue::Float(*value)
    }
}

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
impl TypeWrapper for i64 {
    type Type = i64;
}

impl ScalarTypeWrapper for i64 {
    fn into_scalar(value: &Self::Type) -> ScalarValue {
        ScalarValue::Integer(Some(*value))
    }
}

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
impl TypeWrapper for Option<i64> {
    type Type = Option<i64>;
}

impl ScalarTypeWrapper for Option<i64> {
    fn into_scalar(value: &Self::Type) -> ScalarValue {
        ScalarValue::Integer(*value)
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
impl TypeWrapper for bool {
    type Type = bool;
}

impl ScalarTypeWrapper for bool {
    fn into_scalar(value: &Self::Type) -> ScalarValue {
        ScalarValue::Boolean(Some(*value))
    }
}

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
impl TypeWrapper for Option<bool> {
    type Type = Option<bool>;
}

impl ScalarTypeWrapper for Option<bool> {
    fn into_scalar(value: &Self::Type) -> ScalarValue {
        ScalarValue::Boolean(*value)
    }
}

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
impl TypeWrapper for String {
    type Type = String;
}

impl ScalarTypeWrapper for String {
    fn into_scalar(value: &Self::Type) -> ScalarValue {
        ScalarValue::String(Some(value.clone()))
    }
}

impl Extract<ScalarValue> for String {
    unsafe fn extract(bytes: *const u8) -> ScalarValue {
        let value = unsafe { (*(bytes as *const String)).clone() };
        ScalarValue::String(Some(value))
    }
}

// --- optional string ---
impl TypeWrapper for Option<String> {
    type Type = Option<String>;
}

impl ScalarTypeWrapper for Option<String> {
    fn into_scalar(value: &Self::Type) -> ScalarValue {
        ScalarValue::String(value.clone())
    }
}

impl Extract<ScalarValue> for Option<String> {
    unsafe fn extract(bytes: *const u8) -> ScalarValue {
        let value = unsafe { (*(bytes as *const Option<String>)).clone() };
        ScalarValue::String(value)
    }
}

// --- uom quantities ---
macro_rules! impl_uom {
    ($quantity:path, $unit:path, $unit_trait:path, $conversion_trait:path) => {
        impl TypeWrapper for $unit {
            type Type = $quantity;
        }

        impl ScalarTypeWrapper for $unit {
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
        }

        impl ScalarTypeWrapper for Option<$unit> {
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

with_uom_units!(uom, impl_uom);
