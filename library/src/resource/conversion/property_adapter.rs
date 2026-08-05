use qitech_framework_core::NumericValue;
use qitech_framework_core::ScalarValue;
use qitech_framework_core::ScalarValueTypeMismatchError;
use qitech_framework_core::report::ConstraintViolation;
use qitech_framework_core::report::ParameterConstraints;
use qitech_framework_core::with_uom_units;

use crate::resource::conversion::property_type::PropertyType;

/// Trait to allow defining conversion and extract operations
/// on wrapped units. The best example here is uom which allows us to export
/// a uom Length as millimeter instead of meter (the default with serde feature)
pub trait PropertyAdapter {
    type Type: PropertyType;
    type Input;

    /// converts input when creating into the actual type.
    /// > example: T is millimeter, Input is f64 and output is a Length
    fn convert_input(input: Self::Input) -> Self::Type;

    fn into_scalar(value: Self::Type) -> ScalarValue;
    fn from_scalar(value: ScalarValue) -> Result<Self::Type, ScalarValueTypeMismatchError>;

    fn validate_constraints(
        constraints: &<Self::Type as PropertyType>::Constraints,
        value: &Self::Type,
    ) -> Result<(), ConstraintViolation>;

    fn as_parameter_constraints(
        constraints: &<Self::Type as PropertyType>::Constraints,
    ) -> ParameterConstraints;
}

// --- string ---
impl PropertyAdapter for String {
    type Type = String;
    type Input = String;

    fn convert_input(input: Self::Input) -> Self::Type {
        input
    }

    fn from_scalar(value: ScalarValue) -> Result<Self::Type, ScalarValueTypeMismatchError> {
        match value {
            ScalarValue::String(Some(value)) => Ok(value),
            _ => Err(ScalarValueTypeMismatchError),
        }
    }

    fn into_scalar(value: Self::Type) -> ScalarValue {
        ScalarValue::String(Some(value.clone()))
    }

    fn validate_constraints(
        constraints: &<Self::Type as PropertyType>::Constraints,
        value: &Self::Type,
    ) -> Result<(), ConstraintViolation> {
        if let Some(min) = constraints.min_length
            && value.len() < min
        {
            return Err(ConstraintViolation::StringTooShort {
                length: value.len(),
                min,
            });
        }

        if let Some(max) = constraints.max_length
            && value.len() > max
        {
            return Err(ConstraintViolation::StringTooLong {
                length: value.len(),
                max,
            });
        }

        if let Some((pattern, regex)) = &constraints.pattern
            && !regex.is_match(value)
        {
            return Err(ConstraintViolation::PatternMismatch {
                pattern: (*pattern).to_owned(),
            });
        }

        Ok(())
    }

    fn as_parameter_constraints(
        constraints: &<Self::Type as PropertyType>::Constraints,
    ) -> ParameterConstraints {
        ParameterConstraints::String {
            min_length: constraints.min_length,
            max_length: constraints.max_length,
            pattern: constraints
                .pattern
                .as_ref()
                .map(|(pattern, _)| pattern.to_string()),
            nullable: false,
        }
    }
}

impl PropertyAdapter for Option<String> {
    type Type = Option<String>;
    type Input = Option<String>;

    fn convert_input(input: Self::Input) -> Self::Type {
        input
    }

    fn from_scalar(value: ScalarValue) -> Result<Self::Type, ScalarValueTypeMismatchError> {
        match value {
            ScalarValue::String(value) => Ok(value),
            _ => Err(ScalarValueTypeMismatchError),
        }
    }

    fn into_scalar(value: Self::Type) -> ScalarValue {
        ScalarValue::String(value)
    }

    fn validate_constraints(
        constraints: &<Self::Type as PropertyType>::Constraints,
        value: &Self::Type,
    ) -> Result<(), ConstraintViolation> {
        let value = match value {
            Some(value) => value,
            None => {
                if constraints.allow_none {
                    return Ok(());
                }

                return Err(ConstraintViolation::CannotBeNull {
                    value: ScalarValue::String(None),
                });
            }
        };

        if let Some(min) = constraints.min_length
            && value.len() < min
        {
            return Err(ConstraintViolation::StringTooShort {
                length: value.len(),
                min,
            });
        }

        if let Some(max) = constraints.max_length
            && value.len() > max
        {
            return Err(ConstraintViolation::StringTooLong {
                length: value.len(),
                max,
            });
        }

        if let Some((pattern, regex)) = &constraints.pattern
            && !regex.is_match(value)
        {
            return Err(ConstraintViolation::PatternMismatch {
                pattern: (*pattern).to_owned(),
            });
        }

        Ok(())
    }

    fn as_parameter_constraints(
        constraints: &<Self::Type as PropertyType>::Constraints,
    ) -> ParameterConstraints {
        ParameterConstraints::String {
            min_length: constraints.min_length,
            max_length: constraints.max_length,
            pattern: constraints
                .pattern
                .as_ref()
                .map(|(pattern, _)| (*pattern).to_owned()),
            nullable: constraints.allow_none,
        }
    }
}

// --- bool ---
impl PropertyAdapter for bool {
    type Type = bool;
    type Input = bool;

    fn convert_input(input: Self::Input) -> Self::Type {
        input
    }

    fn from_scalar(value: ScalarValue) -> Result<Self::Type, ScalarValueTypeMismatchError> {
        match value {
            ScalarValue::Boolean(Some(value)) => Ok(value),
            _ => Err(ScalarValueTypeMismatchError),
        }
    }

    fn into_scalar(value: Self::Type) -> ScalarValue {
        ScalarValue::Boolean(Some(value))
    }

    fn validate_constraints(
        constraints: &<Self::Type as PropertyType>::Constraints,
        value: &Self::Type,
    ) -> Result<(), ConstraintViolation> {
        _ = constraints;
        _ = value;
        Ok(())
    }

    fn as_parameter_constraints(
        constraints: &<Self::Type as PropertyType>::Constraints,
    ) -> ParameterConstraints {
        _ = constraints;
        ParameterConstraints::None
    }
}

impl PropertyAdapter for Option<bool> {
    type Type = Option<bool>;
    type Input = Option<bool>;

    fn convert_input(input: Self::Input) -> Self::Type {
        input
    }

    fn from_scalar(value: ScalarValue) -> Result<Self::Type, ScalarValueTypeMismatchError> {
        match value {
            ScalarValue::Boolean(value) => Ok(value),
            _ => Err(ScalarValueTypeMismatchError),
        }
    }

    fn into_scalar(value: Self::Type) -> ScalarValue {
        ScalarValue::Boolean(value)
    }

    fn validate_constraints(
        constraints: &<Self::Type as PropertyType>::Constraints,
        value: &Self::Type,
    ) -> Result<(), ConstraintViolation> {
        _ = constraints;
        _ = value;
        Ok(())
    }

    fn as_parameter_constraints(
        constraints: &<Self::Type as PropertyType>::Constraints,
    ) -> ParameterConstraints {
        _ = constraints;
        ParameterConstraints::None
    }
}

// --- integer ---
impl PropertyAdapter for i64 {
    type Type = i64;
    type Input = i64;

    fn convert_input(input: Self::Input) -> Self::Type {
        input
    }

    fn from_scalar(value: ScalarValue) -> Result<Self::Type, ScalarValueTypeMismatchError> {
        match value {
            ScalarValue::Integer(Some(value)) => Ok(value),
            _ => Err(ScalarValueTypeMismatchError),
        }
    }

    fn into_scalar(value: Self::Type) -> ScalarValue {
        ScalarValue::Integer(Some(value))
    }

    fn validate_constraints(
        constraints: &<Self::Type as PropertyType>::Constraints,
        value: &Self::Type,
    ) -> Result<(), ConstraintViolation> {
        if let Some(min) = constraints.min
            && *value < min
        {
            return Err(ConstraintViolation::BelowMin {
                value: NumericValue::Integer(Some(*value)),
                min: NumericValue::Integer(Some(min)),
            });
        }

        if let Some(max) = constraints.max
            && *value > max
        {
            return Err(ConstraintViolation::AboveMax {
                value: NumericValue::Integer(Some(*value)),
                max: NumericValue::Integer(Some(max)),
            });
        }

        Ok(())
    }

    fn as_parameter_constraints(
        constraints: &<Self::Type as PropertyType>::Constraints,
    ) -> ParameterConstraints {
        ParameterConstraints::Numeric {
            min: NumericValue::Integer(constraints.min),
            max: NumericValue::Integer(constraints.max),
            nullable: false,
        }
    }
}

impl PropertyAdapter for Option<i64> {
    type Type = Option<i64>;
    type Input = Option<i64>;

    fn convert_input(input: Self::Input) -> Self::Type {
        input
    }

    fn from_scalar(value: ScalarValue) -> Result<Self::Type, ScalarValueTypeMismatchError> {
        match value {
            ScalarValue::Integer(value) => Ok(value),
            _ => Err(ScalarValueTypeMismatchError),
        }
    }

    fn into_scalar(value: Self::Type) -> ScalarValue {
        ScalarValue::Integer(value)
    }

    fn validate_constraints(
        constraints: &<Self::Type as PropertyType>::Constraints,
        value: &Self::Type,
    ) -> Result<(), ConstraintViolation> {
        let value = match value {
            Some(value) => value,
            None => {
                if constraints.allow_none {
                    return Ok(());
                }

                return Err(ConstraintViolation::CannotBeNull {
                    value: ScalarValue::Integer(None),
                });
            }
        };

        if let Some(min) = constraints.min
            && *value < min
        {
            return Err(ConstraintViolation::BelowMin {
                value: NumericValue::Integer(Some(*value)),
                min: NumericValue::Integer(Some(min)),
            });
        }

        if let Some(max) = constraints.max
            && *value > max
        {
            return Err(ConstraintViolation::AboveMax {
                value: NumericValue::Integer(Some(*value)),
                max: NumericValue::Integer(Some(max)),
            });
        }

        Ok(())
    }

    fn as_parameter_constraints(
        constraints: &<Self::Type as PropertyType>::Constraints,
    ) -> ParameterConstraints {
        ParameterConstraints::Numeric {
            min: NumericValue::Integer(constraints.min),
            max: NumericValue::Integer(constraints.max),
            nullable: constraints.allow_none,
        }
    }
}

// --- float ---
impl PropertyAdapter for f64 {
    type Type = f64;
    type Input = f64;

    fn convert_input(input: Self::Input) -> Self::Type {
        input
    }

    fn from_scalar(value: ScalarValue) -> Result<Self::Type, ScalarValueTypeMismatchError> {
        match value {
            ScalarValue::Float(Some(value)) => Ok(value),
            _ => Err(ScalarValueTypeMismatchError),
        }
    }

    fn into_scalar(value: Self::Type) -> ScalarValue {
        ScalarValue::Float(Some(value))
    }

    fn validate_constraints(
        constraints: &<Self::Type as PropertyType>::Constraints,
        value: &Self::Type,
    ) -> Result<(), ConstraintViolation> {
        if let Some(min) = constraints.min
            && *value < min
        {
            return Err(ConstraintViolation::BelowMin {
                value: NumericValue::Float(Some(*value)),
                min: NumericValue::Float(Some(min)),
            });
        }

        if let Some(max) = constraints.max
            && *value > max
        {
            return Err(ConstraintViolation::AboveMax {
                value: NumericValue::Float(Some(*value)),
                max: NumericValue::Float(Some(max)),
            });
        }

        Ok(())
    }

    fn as_parameter_constraints(
        constraints: &<Self::Type as PropertyType>::Constraints,
    ) -> ParameterConstraints {
        ParameterConstraints::Numeric {
            min: NumericValue::Float(constraints.min),
            max: NumericValue::Float(constraints.max),
            nullable: false,
        }
    }
}

impl PropertyAdapter for Option<f64> {
    type Type = Option<f64>;
    type Input = Option<f64>;

    fn convert_input(input: Self::Input) -> Self::Type {
        input
    }

    fn from_scalar(value: ScalarValue) -> Result<Self::Type, ScalarValueTypeMismatchError> {
        match value {
            ScalarValue::Float(value) => Ok(value),
            _ => Err(ScalarValueTypeMismatchError),
        }
    }

    fn into_scalar(value: Self::Type) -> ScalarValue {
        ScalarValue::Float(value)
    }

    fn validate_constraints(
        constraints: &<Self::Type as PropertyType>::Constraints,
        value: &Self::Type,
    ) -> Result<(), ConstraintViolation> {
        let value = match value {
            Some(value) => value,
            None => {
                if constraints.allow_none {
                    return Ok(());
                }

                return Err(ConstraintViolation::CannotBeNull {
                    value: ScalarValue::Float(None),
                });
            }
        };

        if let Some(min) = constraints.min
            && *value < min
        {
            return Err(ConstraintViolation::BelowMin {
                value: NumericValue::Float(Some(*value)),
                min: NumericValue::Float(Some(min)),
            });
        }

        if let Some(max) = constraints.max
            && *value > max
        {
            return Err(ConstraintViolation::AboveMax {
                value: NumericValue::Float(Some(*value)),
                max: NumericValue::Float(Some(max)),
            });
        }

        Ok(())
    }

    fn as_parameter_constraints(
        constraints: &<Self::Type as PropertyType>::Constraints,
    ) -> ParameterConstraints {
        ParameterConstraints::Numeric {
            min: NumericValue::Float(constraints.min),
            max: NumericValue::Float(constraints.max),
            nullable: constraints.allow_none,
        }
    }
}

// --- uom ---
macro_rules! impl_uom_unit {
    ($quantity:path, $unit:path, $unit_trait:path, $conversion_trait:path) => {
        impl PropertyAdapter for $unit {
            type Type = $quantity;
            type Input = f64;

            fn convert_input(input: Self::Input) -> Self::Type {
                <$quantity>::new::<$unit>(input)
            }

            fn from_scalar(value: ScalarValue) -> Result<Self::Type, ScalarValueTypeMismatchError> {
                match value {
                    ScalarValue::Float(Some(value)) => Ok(<$quantity>::new::<$unit>(value)),
                    _ => Err(ScalarValueTypeMismatchError),
                }
            }

            fn into_scalar(value: Self::Type) -> ScalarValue {
                ScalarValue::Float(Some(value.get::<$unit>()))
            }

            fn validate_constraints(
                constraints: &<Self::Type as PropertyType>::Constraints,
                value: &Self::Type,
            ) -> Result<(), ConstraintViolation> {
                if let Some(min) = constraints.min {
                    if *value < min {
                        return Err(ConstraintViolation::BelowMin {
                            value: NumericValue::Float(Some(value.get::<$unit>())),
                            min: NumericValue::Float(Some(min.get::<$unit>())),
                        });
                    }
                }

                if let Some(max) = constraints.max {
                    if *value > max {
                        return Err(ConstraintViolation::AboveMax {
                            value: NumericValue::Float(Some(value.get::<$unit>())),
                            max: NumericValue::Float(Some(max.get::<$unit>())),
                        });
                    }
                }

                Ok(())
            }

            fn as_parameter_constraints(
                constraints: &<Self::Type as PropertyType>::Constraints,
            ) -> ParameterConstraints {
                ParameterConstraints::Numeric {
                    min: NumericValue::Float(constraints.min.map(|x| x.get::<$unit>())),
                    max: NumericValue::Float(constraints.max.map(|x| x.get::<$unit>())),
                    nullable: false,
                }
            }
        }

        impl PropertyAdapter for Option<$unit> {
            type Type = Option<$quantity>;
            type Input = Option<f64>;

            fn convert_input(input: Self::Input) -> Self::Type {
                input.map(|x| <$quantity>::new::<$unit>(x))
            }

            fn from_scalar(value: ScalarValue) -> Result<Self::Type, ScalarValueTypeMismatchError> {
                match value {
                    ScalarValue::Float(value) => Ok(value.map(|x| <$quantity>::new::<$unit>(x))),
                    _ => Err(ScalarValueTypeMismatchError),
                }
            }

            fn into_scalar(value: Self::Type) -> ScalarValue {
                ScalarValue::Float(value.map(|x| x.get::<$unit>()))
            }

            fn validate_constraints(
                constraints: &<Self::Type as PropertyType>::Constraints,
                value: &Self::Type,
            ) -> Result<(), ConstraintViolation> {
                let value = match value {
                    Some(value) => value,
                    None => {
                        if constraints.allow_none {
                            return Ok(());
                        }

                        return Err(ConstraintViolation::CannotBeNull {
                            value: ScalarValue::Float(None),
                        });
                    }
                };

                if let Some(min) = constraints.min {
                    if *value < min {
                        return Err(ConstraintViolation::BelowMin {
                            value: NumericValue::Float(Some(value.get::<$unit>())),
                            min: NumericValue::Float(Some(min.get::<$unit>())),
                        });
                    }
                }

                if let Some(max) = constraints.max {
                    if *value > max {
                        return Err(ConstraintViolation::AboveMax {
                            value: NumericValue::Float(Some(value.get::<$unit>())),
                            max: NumericValue::Float(Some(max.get::<$unit>())),
                        });
                    }
                }

                Ok(())
            }

            fn as_parameter_constraints(
                constraints: &<Self::Type as PropertyType>::Constraints,
            ) -> ParameterConstraints {
                ParameterConstraints::Numeric {
                    min: NumericValue::Float(constraints.min.map(|x| x.get::<$unit>())),
                    max: NumericValue::Float(constraints.max.map(|x| x.get::<$unit>())),
                    nullable: constraints.allow_none,
                }
            }
        }
    };
}

with_uom_units!(impl_uom_unit);
