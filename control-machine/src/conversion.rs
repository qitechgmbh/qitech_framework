use std::fmt::Debug;
use control_core::ScalarValue;

pub trait Convertible<T> {
    /// # Safety
    /// `bytes` must point to a valid instance of `Self`.
    unsafe fn convert(bytes: *const u8) -> T;
}

pub trait Bounded { 
    type Bound: Copy + PartialOrd + Debug;
    fn as_bound(&self) -> Self::Bound;
}

impl Bounded for qitech_lib::units::Length {
    type Bound = Self;
    fn as_bound(&self) -> Self::Bound { *self }
}

pub fn in_bounds<T: Bounded>(
    value: &T, 
    min: Option<T::Bound>, 
    max: Option<T::Bound>
) -> bool{
    let value = value.as_bound();

    if let Some(min) = min && value < min{
        return false;
    }

    if let Some(max) = max && value > max{
        return false;
    }

    true
}

#[derive(Debug)]
pub struct BoundsError<T: Debug> {
    value: T,
    min: Option<T>,
    max: Option<T>,
}

impl<T: Debug + std::fmt::Display> std::fmt::Display for BoundsError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.min, &self.max) {
            (Some(min), Some(max)) => {
                write!(f, "value {} out of bounds [{min}, {max}]", self.value)
            }
            (Some(min), None) => write!(f, "value {} below minimum {min}", self.value),
            (None, Some(max)) => write!(f, "value {} above maximum {max}", self.value),
            (None, None) => write!(f, "value {} failed validation", self.value),
        }
    }
}

impl<T: std::fmt::Display + Debug> std::error::Error for BoundsError<T> {}
// --- wrapped ---
pub trait PropertyType {
    type Value: Clone + Default + 'static;
}

pub trait IntoScalar {
    fn into_scalar(value: Self) -> ScalarValue;
}

pub trait ScalarPropertyType: PropertyType + Convertible<ScalarValue> {
    fn into_scalar(value: Self::Value) -> ScalarValue;
}

pub trait FloatPropertyType: PropertyType + Convertible<Option<f64>> {
    fn into_opt_f64(value: Self::Value) -> Option<f64>;
}

macro_rules! impl_float_export {
    ($ty:ty, $expr:expr) => {
        impl Convertible<Option<f64>> for $ty {
            unsafe fn convert(bytes: *const u8) -> Option<f64> {
                let value = unsafe { *(bytes as *const $ty) };
                Some($expr(value))
            }
        }

        impl Convertible<Option<f64>> for Option<$ty> {
            unsafe fn convert(bytes: *const u8) -> Option<f64> {
                let value = unsafe { *(bytes as *const Option<$ty>) };
                value.map($expr)
            }
        }
    };
}

impl_float_export!(bool, |v| if v { 1.0 } else { 0.0 });
impl_float_export!(f64,  |v| v);
impl_float_export!(i64,  |v| v as f64);

// --- string ---
// impl PropertyType for &mut str { type Value = &mut str; }
// impl PropertyType for Option<&mut str> { type Value = &mut str; }

// --- bool ---
impl PropertyType for bool { type Value = bool; }

impl Convertible<ScalarValue> for bool {
    unsafe fn convert(bytes: *const u8) -> ScalarValue {
        let value = unsafe { *(bytes as *const bool) };
        ScalarValue::Boolean { value: Some(value) }
    }
}

impl ScalarPropertyType for bool {
    fn into_scalar(value: Self::Value) -> ScalarValue {
        ScalarValue::Boolean { value: Some(value) }
    }
}

// --- nullable bool ---
impl Convertible<ScalarValue> for Option<bool> {
    unsafe fn convert(bytes: *const u8) -> ScalarValue {
        let value = unsafe { *(bytes as *const Option<bool>) };
        ScalarValue::Boolean { value }
    }
}

// --- f64 ---
impl Convertible<ScalarValue> for f64 {
    unsafe fn convert(bytes: *const u8) -> ScalarValue {
        let value = unsafe { *(bytes as *const f64) };
        ScalarValue::Float { value: Some(value) }
    }
}

// --- nullable f64 ---
impl Convertible<ScalarValue> for Option<f64> {
    unsafe fn convert(bytes: *const u8) -> ScalarValue {
        let value = unsafe { *(bytes as *const Option<f64>) };
        ScalarValue::Float { value }
    }
}

// --- i64 ---
impl Convertible<ScalarValue> for i64 {
    unsafe fn convert(bytes: *const u8) -> ScalarValue {
        let value = unsafe { *(bytes as *const i64) };
        ScalarValue::Integer { value: Some(value) }
    }
}

// --- nullable i64 ---
impl Convertible<ScalarValue> for Option<i64> {
    unsafe fn convert(bytes: *const u8) -> ScalarValue {
        let value = unsafe { *(bytes as *const Option<i64>) };
        ScalarValue::Integer { value }
    }
}

// uom
macro_rules! impl_uom {
    ($quantity:path, $unit:path, $unit_trait:path, $conversion_trait:path) => {
        impl PropertyType for $unit { type Value = $quantity; }

        impl Convertible<ScalarValue> for $unit {
            unsafe fn convert(bytes: *const u8) -> ScalarValue {
                let value = unsafe { *(bytes as *const $quantity) };
                ScalarValue::Float { value: Some(value.get::<$unit>()) }
            }
        }

        impl Convertible<Option<f64>> for $unit {
            unsafe fn convert(bytes: *const u8) -> Option<f64> {
                let value = unsafe { *(bytes as *const $quantity) };
                Some(value.get::<$unit>())
            }
        }

        impl FloatPropertyType for $unit {
            fn into_opt_f64(value: Self::Value) -> Option<f64> {
                Some(value.get::<$unit>())
            }
        }

        impl ScalarPropertyType for $unit {
            fn into_scalar(value: Self::Value) -> ScalarValue {
                ScalarValue::Float { value: Some(value.get::<$unit>()) }
            }
        }

        impl PropertyType for Option<$unit> { type Value = Option<$quantity>; }

        impl Convertible<ScalarValue> for Option<$unit> {
            unsafe fn convert(bytes: *const u8) -> ScalarValue {
                let value = unsafe { *(bytes as *const Option<$quantity>) };
                ScalarValue::Float { value: value.map(|x| x.get::<$unit>()) }
            }
        }

        impl Convertible<Option<f64>> for Option<$unit> {
            unsafe fn convert(bytes: *const u8) -> Option<f64> {
                let value = unsafe { *(bytes as *const Option<$quantity>) };
                value.map(|x| x.get::<$unit>())
            }
        }

        impl FloatPropertyType for Option<$unit> {
            fn into_opt_f64(value: Self::Value) -> Option<f64> {
                value.map(|x| x.get::<$unit>())
            }
        }

        impl ScalarPropertyType for Option<$unit> {
            fn into_scalar(value: Self::Value) -> ScalarValue {
                ScalarValue::Float { value: value.map(|x| x.get::<$unit>()) }
            }
        }
    };
}

with_uom!(impl_uom);
