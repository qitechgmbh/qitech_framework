use control_core::ScalarValue;

pub trait ToScalar {
    fn to_scalar(self) -> ScalarValue;
}

impl ToScalar for f64 {
    fn to_scalar(self) -> ScalarValue {
        ScalarValue::Float(Some(self))
    }
}

impl ToScalar for i64 {
    fn to_scalar(self) -> ScalarValue {
        ScalarValue::IntegerSigned(Some(self))
    }
}

impl ToScalar for u64 {
    fn to_scalar(self) -> ScalarValue {
        ScalarValue::IntegerUnsigned(Some(self))
    }
}

impl ToScalar for bool {
    fn to_scalar(self) -> ScalarValue {
        ScalarValue::Boolean(Some(self))
    }
}

impl ToScalar for String {
    fn to_scalar(self) -> ScalarValue {
        ScalarValue::String(Some(self))
    }
}

impl ToScalar for Option<f64> {
    fn to_scalar(self) -> ScalarValue {
        ScalarValue::Float(self)
    }
}

impl ToScalar for Option<i64> {
    fn to_scalar(self) -> ScalarValue {
        ScalarValue::IntegerSigned(self)
    }
}

impl ToScalar for Option<u64> {
    fn to_scalar(self) -> ScalarValue {
        ScalarValue::IntegerUnsigned(self)
    }
}

impl ToScalar for Option<bool> {
    fn to_scalar(self) -> ScalarValue {
        ScalarValue::Boolean(self)
    }
}

impl ToScalar for Option<String> {
    fn to_scalar(self) -> ScalarValue {
        ScalarValue::String(self)
    }
}