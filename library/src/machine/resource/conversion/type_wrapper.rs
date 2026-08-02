use qitech_framework_core::ScalarValue;
use qitech_framework_core::report::MachineConfigPropertyConstraints;
use qitech_framework_core::with_uom_units;

use crate::machine::resource::constraints::NumericConfigPropertyConstraints;
use crate::machine::resource::constraints::StringConfigPropertyConstraints;

/// Trait to allow defining conversion and extract operations
/// on wrapped units. The best example here is uom which allows us to export
/// a uom Length as millimeter instead of meter (the default with serde feature)
pub trait TypeWrapper {
    type Type: Clone + 'static;
    type Input;
    type Constraints;

    /// converts input when creating into the actual type.
    /// > example: T is millimeter, Input is f64 and output is a Length
    fn convert_input(input: Self::Input) -> Self::Type;

    fn into_scalar(value: Self::Type) -> ScalarValue;
    fn from_scalar(value: ScalarValue) -> Option<Self::Type>;

    /// converts the custom constraints into the generic form
    fn into_constraints(constraints: Self::Constraints) -> MachineConfigPropertyConstraints;
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

    fn into_scalar(value: Self::Type) -> ScalarValue {
        ScalarValue::String(Some(value.clone()))
    }

    fn into_constraints(constraints: Self::Constraints) -> MachineConfigPropertyConstraints {
        MachineConfigPropertyConstraints::String {
            min_length: constraints.min_length,
            max_length: constraints.max_length,
        }
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

    fn into_scalar(value: Self::Type) -> ScalarValue {
        ScalarValue::String(value.clone())
    }

    fn into_constraints(constraints: Self::Constraints) -> MachineConfigPropertyConstraints {
        MachineConfigPropertyConstraints::String {
            min_length: constraints.min_length,
            max_length: constraints.max_length,
        }
    }
}

// --- bool ---
impl TypeWrapper for bool {
    type Type = bool;
    type Input = bool;
    type Constraints = ();

    fn convert_input(input: Self::Input) -> Self::Type {
        input
    }

    fn from_scalar(value: ScalarValue) -> Option<Self::Type> {
        match value {
            ScalarValue::Boolean(Some(value)) => Some(value),
            _ => None,
        }
    }

    fn into_scalar(value: Self::Type) -> ScalarValue {
        ScalarValue::Boolean(Some(value))
    }

    fn into_constraints(constraints: Self::Constraints) -> MachineConfigPropertyConstraints {
        _ = constraints;
        MachineConfigPropertyConstraints::None
    }
}

impl TypeWrapper for Option<bool> {
    type Type = Option<bool>;
    type Input = Option<bool>;
    type Constraints = ();

    fn convert_input(input: Self::Input) -> Self::Type {
        input
    }

    fn from_scalar(value: ScalarValue) -> Option<Self::Type> {
        match value {
            ScalarValue::Boolean(value) => Some(value),
            _ => None,
        }
    }

    fn into_scalar(value: Self::Type) -> ScalarValue {
        ScalarValue::Boolean(value)
    }

    fn into_constraints(constraints: Self::Constraints) -> MachineConfigPropertyConstraints {
        _ = constraints;
        MachineConfigPropertyConstraints::None
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

    fn into_scalar(value: Self::Type) -> ScalarValue {
        ScalarValue::Integer(Some(value))
    }

    fn into_constraints(constraints: Self::Constraints) -> MachineConfigPropertyConstraints {
        MachineConfigPropertyConstraints::Integer {
            min: constraints.min,
            max: constraints.max,
        }
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

    fn into_scalar(value: Self::Type) -> ScalarValue {
        ScalarValue::Integer(value)
    }

    fn into_constraints(constraints: Self::Constraints) -> MachineConfigPropertyConstraints {
        MachineConfigPropertyConstraints::Integer {
            min: constraints.min,
            max: constraints.max,
        }
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

    fn into_scalar(value: Self::Type) -> ScalarValue {
        ScalarValue::Float(Some(value.clone()))
    }

    fn into_constraints(constraints: Self::Constraints) -> MachineConfigPropertyConstraints {
        MachineConfigPropertyConstraints::Float {
            min: constraints.min,
            max: constraints.max,
        }
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

    fn into_scalar(value: Self::Type) -> ScalarValue {
        ScalarValue::Float(value.clone())
    }

    fn into_constraints(constraints: Self::Constraints) -> MachineConfigPropertyConstraints {
        MachineConfigPropertyConstraints::Float {
            min: constraints.min,
            max: constraints.max,
        }
    }
}

// --- uom ---
macro_rules! impl_uom_unit {
    ($quantity:path, $unit:path, $unit_trait:path, $conversion_trait:path) => {
        impl TypeWrapper for $unit {
            type Type = $quantity;
            type Input = f64;
            type Constraints = NumericConfigPropertyConstraints<$quantity>;

            fn convert_input(input: f64) -> $quantity {
                <$quantity>::new::<$unit>(input)
            }

            fn from_scalar(value: ScalarValue) -> Option<Self::Type> {
                match value {
                    ScalarValue::Float(Some(x)) => Some(<$quantity>::new::<$unit>(x)),
                    _ => None,
                }
            }

            fn into_scalar(value: Self::Type) -> ScalarValue {
                ScalarValue::Float(Some(value.get::<$unit>()))
            }

            fn into_constraints(
                constraints: Self::Constraints,
            ) -> MachineConfigPropertyConstraints {
                MachineConfigPropertyConstraints::Float {
                    min: constraints.min.map(|x| x.get::<$unit>()),
                    max: constraints.max.map(|x| x.get::<$unit>()),
                }
            }
        }

        impl TypeWrapper for Option<$unit> {
            type Type = Option<$quantity>;
            type Input = Option<f64>;
            type Constraints = NumericConfigPropertyConstraints<$quantity>;

            fn convert_input(input: Option<f64>) -> Option<$quantity> {
                input.map(|x| <$quantity>::new::<$unit>(x))
            }

            fn from_scalar(value: ScalarValue) -> Option<Self::Type> {
                match value {
                    ScalarValue::Float(x) => Some(x.map(|x| <$quantity>::new::<$unit>(x))),
                    _ => None,
                }
            }

            fn into_scalar(value: Self::Type) -> ScalarValue {
                ScalarValue::Float(value.map(|x| x.get::<$unit>()))
            }

            fn into_constraints(
                constraints: Self::Constraints,
            ) -> MachineConfigPropertyConstraints {
                MachineConfigPropertyConstraints::Float {
                    min: constraints.min.map(|x| x.get::<$unit>()),
                    max: constraints.max.map(|x| x.get::<$unit>()),
                }
            }
        }
    };
}

with_uom_units!(impl_uom_unit);
