use std::borrow::Cow;
use std::fmt::Debug;

use qitech_framework_common::ScalarValue;
use qitech_framework_common::with_uom_quantities;
use qitech_framework_common::with_uom_units;
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

    fn convert_input(input: Self::Input) -> Self::Type;
}

/// Specialized TypeWrapper for dealing with scalar values.
/// Used by ConfigPropertyManager and StatePropertyManager for registration.
pub trait ScalarTypeWrapper: TypeWrapper + Clone + Extract<ScalarValue> + 'static {
    fn into_scalar(value: &Self::Type) -> ScalarValue;
}

// --- type wrapper ---
macro_rules! simple_type_wrapper {
    ($type:ty) => {
        impl TypeWrapper for $type {
            type Type = $type;
            type Input = $type;

            fn convert_input(input: Self::Input) -> Self::Type {
                input
            }
        }

        impl TypeWrapper for Option<$type> {
            type Type = Option<$type>;
            type Input = Option<$type>;

            fn convert_input(input: Self::Input) -> Self::Type {
                input
            }
        }
    };
}

simple_type_wrapper!(f64);
simple_type_wrapper!(i64);
simple_type_wrapper!(bool);

impl TypeWrapper for String {
    type Type = String;
    type Input = &'static str;

    fn convert_input(input:  &'static str) -> String {
        input.to_string()
    }
}

impl TypeWrapper for Option<String> {
    type Type = Option<String>;
    type Input = Option<&'static str>;

    fn convert_input(input:  Option<&'static str>) -> Option<String> {
        input.map(|x| x.to_string())
    }
}

// --- float ---
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
impl ScalarTypeWrapper for String {
    fn into_scalar(value: &Self::Type) -> ScalarValue {
        ScalarValue::String(Some(Cow::Owned(value.clone())))
    }
}

impl Extract<ScalarValue> for String {
    unsafe fn extract(bytes: *const u8) -> ScalarValue {
        let value = unsafe { (*(bytes as *const String)).clone() };
        ScalarValue::String(Some(Cow::Owned(value.clone())))
    }
}

impl BoundedMeta for String {
    type Bound = usize;

    fn as_bound(&self) -> Option<Self::Bound> {
        Some(self.len())
    }
}

// --- optional string ---
impl ScalarTypeWrapper for Option<String> {
    fn into_scalar(value: &Self::Type) -> ScalarValue {
        ScalarValue::String(value.clone().map(Cow::Owned))
    }
}

impl Extract<ScalarValue> for Option<String> {
    unsafe fn extract(bytes: *const u8) -> ScalarValue {
        let value = unsafe { (*(bytes as *const Option<String>)).clone() };
        ScalarValue::String(value.clone().map(Cow::Owned))
    }
}

// --- uom quantities ---
macro_rules! impl_uom_unit {
    ($quantity:path, $unit:path, $unit_trait:path, $conversion_trait:path) => {
        impl TypeWrapper for $unit {
            type Type = $quantity;
            type Input = f64;

            fn convert_input(input: f64) -> $quantity {
                <$quantity>::new::<$unit>(input)
            }
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
            type Input = Option<f64>;

            fn convert_input(input: Option<f64>) -> Option<$quantity> {
                input.map(|x| <$quantity>::new::<$unit>(x))
            }
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

with_uom_units!(uom, impl_uom_unit);

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

with_uom_quantities!(uom, impl_uom_quantity);