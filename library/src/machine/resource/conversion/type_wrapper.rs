use qitech_framework_core::ScalarValue;
use qitech_framework_core::report::MachineConfigPropertyConstraints;

use crate::machine::resource::constraints::BooleanConfigPropertyConstraints;
use crate::machine::resource::constraints::NumericConfigPropertyConstraints;
use crate::machine::resource::constraints::StringConfigPropertyConstraints;

/// Trait to allow defining conversion and extract operations
/// on wrapped units. The best example here is uom which allows us to export
/// a uom Length as millimeter instead of meter (the default with serde feature)
pub trait TypeWrapper {
    type Type: Clone + 'static;
    type Input;
    type Constraints: Into<MachineConfigPropertyConstraints>;

    fn convert_input(input: Self::Input) -> Self::Type;
    fn into_scalar(value: &Self::Type) -> ScalarValue;
    fn from_scalar(value: ScalarValue) -> Option<Self::Type>;
}

// --- string ---
impl TypeWrapper for String {
    type Type = String;
    type Input = String;
    type Constraints = StringConfigPropertyConstraints;

    fn convert_input(input: Self::Input) -> Self::Type {
        input
    }

    fn from_scalar(value: ScalarValue) -> Option<Self::Type> {
        match value {
            ScalarValue::String(Some(value)) => Some(value),
            _ => None,
        }
    }

    fn into_scalar(value: &Self::Type) -> ScalarValue {
        ScalarValue::String(Some(value.clone()))
    }
}

impl TypeWrapper for Option<String> {
    type Type = Option<String>;
    type Input = Option<String>;
    type Constraints = StringConfigPropertyConstraints;

    fn convert_input(input: Self::Input) -> Self::Type {
        input
    }

    fn from_scalar(value: ScalarValue) -> Option<Self::Type> {
        match value {
            ScalarValue::String(value) => Some(value),
            _ => None,
        }
    }

    fn into_scalar(value: &Self::Type) -> ScalarValue {
        ScalarValue::String(value.clone())
    }
}

// --- bool ---
impl TypeWrapper for bool {
    type Type = bool;
    type Input = bool;
    type Constraints = BooleanConfigPropertyConstraints;

    fn convert_input(input: Self::Input) -> Self::Type {
        input
    }

    fn from_scalar(value: ScalarValue) -> Option<Self::Type> {
        match value {
            ScalarValue::Boolean(Some(value)) => Some(value),
            _ => None,
        }
    }

    fn into_scalar(value: &Self::Type) -> ScalarValue {
        ScalarValue::Boolean(Some(*value))
    }
}

impl TypeWrapper for Option<bool> {
    type Type = Option<bool>;
    type Input = Option<bool>;
    type Constraints = BooleanConfigPropertyConstraints;

    fn convert_input(input: Self::Input) -> Self::Type {
        input
    }

    fn from_scalar(value: ScalarValue) -> Option<Self::Type> {
        match value {
            ScalarValue::Boolean(value) => Some(value),
            _ => None,
        }
    }

    fn into_scalar(value: &Self::Type) -> ScalarValue {
        ScalarValue::Boolean(*value)
    }
}

// --- float ---
impl TypeWrapper for f64 {
    type Type = f64;
    type Input = f64;
    type Constraints = NumericConfigPropertyConstraints<f64>;

    fn convert_input(input: Self::Input) -> Self::Type {
        input
    }

    fn from_scalar(value: ScalarValue) -> Option<Self::Type> {
        match value {
            ScalarValue::Float(Some(value)) => Some(value),
            _ => None,
        }
    }

    fn into_scalar(value: &Self::Type) -> ScalarValue {
        ScalarValue::Float(Some(value.clone()))
    }
}

impl TypeWrapper for Option<f64> {
    type Type = Option<f64>;
    type Input = Option<f64>;
    type Constraints = NumericConfigPropertyConstraints<f64>;

    fn convert_input(input: Self::Input) -> Self::Type {
        input
    }

    fn from_scalar(value: ScalarValue) -> Option<Self::Type> {
        match value {
            ScalarValue::Float(value) => Some(value),
            _ => None,
        }
    }

    fn into_scalar(value: &Self::Type) -> ScalarValue {
        ScalarValue::Float(value.clone())
    }
}

// --- integer ---
impl TypeWrapper for i64 {
    type Type = i64;
    type Input = i64;
    type Constraints = NumericConfigPropertyConstraints<i64>;

    fn convert_input(input: Self::Input) -> Self::Type {
        input
    }

    fn from_scalar(value: ScalarValue) -> Option<Self::Type> {
        match value {
            ScalarValue::Integer(Some(value)) => Some(value),
            _ => None,
        }
    }

    fn into_scalar(value: &Self::Type) -> ScalarValue {
        ScalarValue::Integer(Some(*value))
    }
}

impl TypeWrapper for Option<i64> {
    type Type = Option<i64>;
    type Input = Option<i64>;
    type Constraints = NumericConfigPropertyConstraints<i64>;

    fn convert_input(input: Self::Input) -> Self::Type {
        input
    }

    fn from_scalar(value: ScalarValue) -> Option<Self::Type> {
        match value {
            ScalarValue::Integer(Some(value)) => Some(Some(value)),
            _ => None,
        }
    }

    fn into_scalar(value: &Self::Type) -> ScalarValue {
        ScalarValue::Integer(*value)
    }
}
