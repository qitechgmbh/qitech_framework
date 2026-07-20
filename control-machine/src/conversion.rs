use std::fmt::Debug;
use control_core::ScalarValue;

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
pub trait Wrapped { type Inner; }

pub trait WrappedIntoOptionalF64
where 
    Self: Wrapped,
{
    fn into_opt_f64(value: Self::Inner) -> Option<f64>;
}

pub trait WrappedTryFromOptionalF64
where 
    Self: Wrapped,
{
    fn try_from_opt_f64(value: Option<f64>) -> Option<Self::Inner>;
}

pub trait WrappedIntoScalar
where 
    Self: Wrapped,
{
    fn into_scalar(value: Self::Inner) -> ScalarValue;
}

pub trait NonNullableFloatWrapper
where 
    Self: Wrapped,
{
    fn from_f64(value: f64) -> Self::Inner;
    fn into_f64(value: Self::Inner) -> f64;
}

pub trait NullableFloatWrapper
where 
    Self: Wrapped + WrappedIntoOptionalF64,
    Self::Inner: Copy
{
    fn from_opt_f64(value: Option<f64>) -> Self::Inner;
}

// bool
impl Wrapped for bool { type Inner = bool; }

impl WrappedIntoScalar for bool {
    fn into_scalar(value: Self::Inner) -> ScalarValue {
        ScalarValue::Boolean { value: Some(value) }
    }
}

impl Wrapped for Option<bool> { type Inner = Option<bool>; }

impl WrappedIntoScalar for Option<bool> {
    fn into_scalar(value: Self::Inner) -> ScalarValue {
        ScalarValue::Boolean { value }
    }
}

// f64
impl Wrapped for f64 { type Inner = f64; }

impl WrappedIntoScalar for f64 {
    fn into_scalar(value: Self::Inner) -> ScalarValue {
        ScalarValue::Float { value: Some(value) }
    }
}

impl NonNullableFloatWrapper for f64 {
    fn from_f64(value: f64) -> f64 { value }
    fn into_f64(value: f64) -> f64 { value }
}

impl WrappedIntoOptionalF64 for f64 {
    fn into_opt_f64(value: f64) -> Option<f64> { Some(value) }
}

impl Wrapped for Option<f64> { type Inner = Option<f64>; }
impl NullableFloatWrapper for Option<f64> {
    fn from_opt_f64(value: Option<f64>) -> Self::Inner { value }
}

impl WrappedIntoOptionalF64 for Option<f64> {
    fn into_opt_f64(value: Self::Inner) -> Option<f64> { value }
}

// i64
impl Wrapped for i64 { type Inner = i64; }

impl WrappedIntoScalar for i64 {
    fn into_scalar(value: Self::Inner) -> ScalarValue {
        ScalarValue::Integer { value: Some(value) }
    }
}

impl NonNullableFloatWrapper for i64 {
    fn from_f64(value: f64) -> i64 { value as i64 }
    fn into_f64(value: i64) -> f64 { value as f64 }
}

impl WrappedIntoOptionalF64 for i64 {
    fn into_opt_f64(value: i64) -> Option<f64> { Some(value as f64) }
}

impl Wrapped for Option<i64> { type Inner = Option<i64>; }
impl NullableFloatWrapper for Option<i64> {
    fn from_opt_f64(value: Option<f64>) -> Self::Inner { value.map(|x| x as i64) }
}

impl WrappedIntoOptionalF64 for Option<i64> {
    fn into_opt_f64(value: Self::Inner) -> Option<f64> { value.map(|x| x as f64) }
}

// uom
macro_rules! impl_uom {
    ($quantity:path, $unit:path, $unit_trait:path, $conversion_trait:path) => {
        impl Wrapped for $unit { type Inner = $quantity; }

        impl WrappedIntoScalar for $unit {
            fn into_scalar(value: Self::Inner) -> ScalarValue {
                ScalarValue::Float { value: Self::into_opt_f64(value) }
            }
        }

        impl NonNullableFloatWrapper for $unit {
            fn from_f64(value: f64) -> Self::Inner { <Self::Inner>::new::<Self>(value) }
            fn into_f64(value: Self::Inner) -> f64 { value.get::<Self>() }
        }

        impl WrappedIntoOptionalF64 for $unit {
            fn into_opt_f64(value: Self::Inner) -> Option<f64> { Some(value.get::<$unit>()) }
        }

        impl WrappedTryFromOptionalF64 for $unit {
            fn try_from_opt_f64(value: Option<f64>) -> Option<Self::Inner> { 
                Some(<Self::Inner>::new::<Self>(value?))
            }
        }

        impl Wrapped for Option<$unit> { type Inner = Option<$quantity>; }

        impl WrappedIntoScalar for Option<$unit> {
            fn into_scalar(value: Self::Inner) -> ScalarValue {
                ScalarValue::Float { value: Self::into_opt_f64(value) }
            }
        }

        impl NullableFloatWrapper for Option<$unit> {
            fn from_opt_f64(value: Option<f64>) -> Self::Inner { value.map(|x| <$quantity>::new::<$unit>(x)) }
        }

        impl WrappedIntoOptionalF64 for Option<$unit> {
            fn into_opt_f64(value: Self::Inner) -> Option<f64> { value.map(|x| x.get::<$unit>()) }
        }

        impl WrappedTryFromOptionalF64 for Option<$unit> {
            fn try_from_opt_f64(value: Option<f64>) -> Option<Self::Inner> {
                Some(value.map(|x| <$quantity>::new::<$unit>(x)))
            }
        }
    };
}

with_uom!(impl_uom);