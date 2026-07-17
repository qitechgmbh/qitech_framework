use control_core::ScalarValue;

pub trait ToScalar {
    fn to_scalar(self) -> ScalarValue;
}

impl ToScalar for f64 {
    fn to_scalar(self) -> ScalarValue {
        ScalarValue::Float { value: Some(self) }
    }
}

impl ToScalar for i64 {
    fn to_scalar(self) -> ScalarValue {
        ScalarValue::Integer { value: Some(self) }
    }
}

impl ToScalar for bool {
    fn to_scalar(self) -> ScalarValue {
        ScalarValue::Boolean { value: Some(self) }
    }
}

impl ToScalar for String {
    fn to_scalar(self) -> ScalarValue {
        ScalarValue::String { value: Some(self) }
    }
}

impl ToScalar for Option<f64> {
    fn to_scalar(self) -> ScalarValue {
        ScalarValue::Float { value: self }
    }
}

impl ToScalar for Option<i64> {
    fn to_scalar(self) -> ScalarValue {
        ScalarValue::Integer { value: self }
    }
}

impl ToScalar for Option<bool> {
    fn to_scalar(self) -> ScalarValue {
        ScalarValue::Boolean { value: self }
    }
}

impl ToScalar for Option<String> {
    fn to_scalar(self) -> ScalarValue {
        ScalarValue::String { value: self }
    }
}