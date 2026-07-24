use std::fmt::Debug;

use qitech_framework_common::{ScalarValue, with_uom_units};

use crate::machine::error::BoundsError;
use crate::uom;

pub trait Extract<T> {
    unsafe fn extract(bytes: *const u8) -> T;
}

pub trait TypeWrapper {
    type Type: Clone + 'static;
}

// --- float ---
impl TypeWrapper for f64 {
    type Type = f64;
}

impl ScalarTypeWrapper for f64 {
    fn into_string(value: &Self::Type) -> String {
        value.to_string()
    }

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

// ---
impl Extract<Option<f64>> for Option<f64> {
    unsafe fn extract(bytes: *const u8) -> Option<f64> {
        let value = unsafe { *(bytes as *const Option<f64>) };
        value
    }
}

pub trait BoundedMeta {
    type Bound: Copy + PartialOrd + Debug;

    fn validate(
        &self,
        min: Option<Self::Bound>,
        max: Option<Self::Bound>,
    ) -> Result<Self::Bound, BoundsError>;
}

pub trait ScalarTypeWrapper: TypeWrapper + Clone + Extract<ScalarValue> + 'static {
    fn into_string(value: &Self::Type) -> String;
    fn into_scalar(value: &Self::Type) -> ScalarValue;
}


macro_rules! impl_uom {
    ($quantity:path, $unit:path, $unit_trait:path, $conversion_trait:path) => {
        impl TypeWrapper for $unit {
            type Type = $quantity;
        }

        impl ScalarTypeWrapper for $unit {
            fn into_string(value: &Self::Type) -> String {
                value.get::<$unit>().to_string()
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
        }

        impl ScalarTypeWrapper for Option<$unit> {
            fn into_string(value: &Self::Type) -> String {
                todo!()
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
    }
}

with_uom_units!(uom, impl_uom);