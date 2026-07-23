use std::fmt::Debug;
use qitech_framework_common::ScalarValue;

pub trait Extract<T> {
    unsafe fn extract(bytes: *const u8) -> T;
}

pub trait TypeWrapper {
    type Type: Clone + 'static;
}

// --- float ---
impl TypeWrapper  for f64 {
    type Type = f64;
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
    fn as_bound(&self) -> Self::Bound;
}

pub trait ScalarTypeWrapper: TypeWrapper + Extract<ScalarValue> + 'static {
    fn into_string(value: &Self::Type) -> String;
}
