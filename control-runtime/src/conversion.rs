use qitech_lib::units::{Length, length::{millimeter, Unit}};

pub trait QuantityInfo {
    type Quantity;
}

pub trait QuantityOf {
    type Quantity;
}

pub trait FromF64 {
    type Output;
    fn from_f64(value: f64) -> Self::Output;
}

pub trait ToF64 {
    type Input;
    fn to_f64(value: Self::Input) -> f64;
}

// --- float ---
impl FromF64 for f64 {
    type Output = f64;

    fn from_f64(value: f64) -> Self::Output {
        value
    }
}

impl ToF64 for f64 {
    type Input = f64;
    
    fn to_f64(value: f64) -> f64 {
        value
    }
}

// --- millimeter ---
impl QuantityOf for millimeter {
    type Quantity = Length;
}

pub trait FloatRepr {
    type Value: Copy;
    fn from_f64(value: f64) -> Self::Value;
    fn to_f64(value: Self::Value) -> f64;
}

impl FloatRepr for millimeter {
    type Value = Length;

    fn from_f64(value: f64) -> Self::Value {
        Length::new::<millimeter>(value)
    }

    fn to_f64(value: Length) -> f64 {
        value.get::<millimeter>()
    }
}

pub trait UomRepr {
    type Value: Copy;
    fn from_f64(value: f64) -> Self::Value;
    fn to_f64(value: Self::Value) -> f64;
}