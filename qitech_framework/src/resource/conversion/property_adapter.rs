use qitech_framework_core::ScalarValue;
use qitech_framework_core::ScalarValueTypeMismatchError;
use qitech_framework_core::report::ConstraintViolationError;
use qitech_framework_core::report::Constraints;
use qitech_framework_core::schema::FloatSemantic;
use qitech_framework_core::schema::MeasurementDefinition;
use qitech_framework_core::schema::MeasurementKind;
use qitech_framework_core::schema::ScalarPropertyDefinition;
use qitech_framework_core::schema::ScalarPropertyKind;
use qitech_framework_core::with_uom_units;

use crate::resource::constraints::NumericConstraints;
use crate::resource::conversion::PropertyType;

/// Trait to allow defining conversion and extract operations
/// on wrapped units. The best example here is uom which allows us to export
/// a uom Length as millimeter instead of meter (the default with serde feature)
pub trait PropertyAdapter: 'static {
    type Type: PropertyType;
    type Input;

    /// converts input when creating into the actual type.
    /// > example: T is millimeter, Input is f64 and output is a Length
    fn convert_input(input: Self::Input) -> Self::Type;

    fn into_scalar(value: Self::Type) -> ScalarValue;
    fn from_scalar(value: ScalarValue) -> Result<Self::Type, ScalarValueTypeMismatchError>;

    fn validate_scalar_property_definition(
        definition: &ScalarPropertyDefinition,
        ignore_nullable: bool,
    ) -> bool;

    fn validate_measurement_definition(
        definition: &MeasurementDefinition,
        ignore_nullable: bool,
    ) -> bool;

    fn apply_constraints(
        constraints: &<Self::Type as PropertyType>::Constraints,
        value: &Self::Type,
    ) -> Result<(), ConstraintViolationError>;

    fn as_constraints(constraints: &<Self::Type as PropertyType>::Constraints) -> Constraints;
}

// --- string ---
impl<const CAPACITY: usize> PropertyAdapter for heapless::String<CAPACITY> {
    type Type = heapless::String<CAPACITY>;
    type Input = heapless::String<CAPACITY>;

    fn convert_input(input: Self::Input) -> Self::Type {
        input
    }

    fn from_scalar(value: ScalarValue) -> Result<Self::Type, ScalarValueTypeMismatchError> {
        match value {
            ScalarValue::String(value) => {
                // TODO: return error our of bounds
                let mut out = Self::Type::default();
                out.push_str(&value).unwrap();
                Ok(out)
            }
            _ => Err(ScalarValueTypeMismatchError),
        }
    }

    fn into_scalar(value: Self::Type) -> ScalarValue {
        ScalarValue::String(value.to_string())
    }

    fn validate_scalar_property_definition(
        definition: &ScalarPropertyDefinition,
        ignore_nullable: bool,
    ) -> bool {
        if !ignore_nullable && definition.nullable {
            return false;
        }

        matches!(definition.kind, ScalarPropertyKind::String)
    }

    fn validate_measurement_definition(
        definition: &MeasurementDefinition,
        ignore_nullable: bool,
    ) -> bool {
        if !ignore_nullable && definition.nullable {
            return false;
        }

        false
    }

    fn apply_constraints(
        constraints: &<Self::Type as PropertyType>::Constraints,
        value: &Self::Type,
    ) -> Result<(), ConstraintViolationError> {
        if let Some(min) = constraints.min_length
            && value.len() < min
        {
            return Err(ConstraintViolationError::StringTooShort {
                length: value.len(),
                min,
            });
        }

        if let Some((pattern, regex)) = &constraints.pattern
            && !regex.is_match(value)
        {
            return Err(ConstraintViolationError::PatternMismatch {
                pattern: (*pattern).to_owned(),
            });
        }

        Ok(())
    }

    fn as_constraints(constraints: &<Self::Type as PropertyType>::Constraints) -> Constraints {
        Constraints::String {
            min_length: constraints.min_length,
            max_length: CAPACITY,
            pattern: constraints
                .pattern
                .as_ref()
                .map(|(pattern, _)| pattern.to_string()),
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
            ScalarValue::Boolean(value) => Ok(value),
            _ => Err(ScalarValueTypeMismatchError),
        }
    }

    fn into_scalar(value: Self::Type) -> ScalarValue {
        ScalarValue::Boolean(value)
    }

    fn validate_scalar_property_definition(
        definition: &ScalarPropertyDefinition,
        ignore_nullable: bool,
    ) -> bool {
        if !ignore_nullable && definition.nullable {
            return false;
        }

        matches!(definition.kind, ScalarPropertyKind::Boolean)
    }

    fn validate_measurement_definition(
        definition: &MeasurementDefinition,
        ignore_nullable: bool,
    ) -> bool {
        if !ignore_nullable && definition.nullable {
            return false;
        }

        matches!(definition.kind, MeasurementKind::Boolean)
    }

    fn apply_constraints(
        constraints: &<Self::Type as PropertyType>::Constraints,
        value: &Self::Type,
    ) -> Result<(), ConstraintViolationError> {
        _ = constraints;
        _ = value;
        Ok(())
    }

    fn as_constraints(constraints: &<Self::Type as PropertyType>::Constraints) -> Constraints {
        _ = constraints;
        Constraints::None
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
            ScalarValue::Integer(value) => Ok(value),
            _ => Err(ScalarValueTypeMismatchError),
        }
    }

    fn into_scalar(value: Self::Type) -> ScalarValue {
        ScalarValue::Integer(value)
    }

    fn validate_scalar_property_definition(
        definition: &ScalarPropertyDefinition,
        ignore_nullable: bool,
    ) -> bool {
        if !ignore_nullable && definition.nullable {
            return false;
        }

        matches!(definition.kind, ScalarPropertyKind::Integer)
    }

    fn validate_measurement_definition(
        definition: &MeasurementDefinition,
        ignore_nullable: bool,
    ) -> bool {
        if !ignore_nullable && definition.nullable {
            return false;
        }

        matches!(definition.kind, MeasurementKind::Integer { .. })
    }

    fn apply_constraints(
        constraints: &<Self::Type as PropertyType>::Constraints,
        value: &Self::Type,
    ) -> Result<(), ConstraintViolationError> {
        if let Some(min) = constraints.min
            && *value < min
        {
            return Err(ConstraintViolationError::BelowMin {
                value: ScalarValue::Integer(min),
                min: ScalarValue::Integer(*value),
            });
        }

        if let Some(max) = constraints.max
            && *value > max
        {
            return Err(ConstraintViolationError::AboveMax {
                value: ScalarValue::Integer(*value),
                max: ScalarValue::Integer(max),
            });
        }

        Ok(())
    }

    fn as_constraints(constraints: &<Self::Type as PropertyType>::Constraints) -> Constraints {
        Constraints::Numeric {
            min: constraints
                .min
                .map_or(ScalarValue::Null, ScalarValue::Integer),
            max: constraints
                .max
                .map_or(ScalarValue::Null, ScalarValue::Integer),
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
            ScalarValue::Float(value) => Ok(value),
            _ => Err(ScalarValueTypeMismatchError),
        }
    }

    fn into_scalar(value: Self::Type) -> ScalarValue {
        ScalarValue::Float(value)
    }

    fn validate_scalar_property_definition(
        definition: &ScalarPropertyDefinition,
        ignore_nullable: bool,
    ) -> bool {
        if !ignore_nullable && definition.nullable {
            return false;
        }

        !matches!(
            definition.kind,
            ScalarPropertyKind::Float {
                semantic: FloatSemantic::Quantity(_)
            }
        )
    }

    fn validate_measurement_definition(
        definition: &MeasurementDefinition,
        ignore_nullable: bool,
    ) -> bool {
        if !ignore_nullable && definition.nullable {
            return false;
        }

        !matches!(
            definition.kind,
            MeasurementKind::Float {
                semantic: FloatSemantic::Quantity(_),
                ..
            }
        )
    }

    fn apply_constraints(
        constraints: &<Self::Type as PropertyType>::Constraints,
        value: &Self::Type,
    ) -> Result<(), ConstraintViolationError> {
        if let Some(min) = constraints.min
            && *value < min
        {
            return Err(ConstraintViolationError::BelowMin {
                value: ScalarValue::Float(min),
                min: ScalarValue::Float(*value),
            });
        }

        if let Some(max) = constraints.max
            && *value > max
        {
            return Err(ConstraintViolationError::AboveMax {
                value: ScalarValue::Float(*value),
                max: ScalarValue::Float(max),
            });
        }

        Ok(())
    }

    fn as_constraints(constraints: &<Self::Type as PropertyType>::Constraints) -> Constraints {
        Constraints::Numeric {
            min: constraints
                .min
                .map_or(ScalarValue::Null, ScalarValue::Float),
            max: constraints
                .max
                .map_or(ScalarValue::Null, ScalarValue::Float),
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
                    ScalarValue::Float(value) => Ok(<$quantity>::new::<$unit>(value)),
                    _ => Err(ScalarValueTypeMismatchError),
                }
            }

            fn into_scalar(value: Self::Type) -> ScalarValue {
                ScalarValue::Float(value.get::<$unit>())
            }

            // TODO: implement
            fn validate_scalar_property_definition(
                definition: &ScalarPropertyDefinition,
                ignore_nullable: bool,
            ) -> bool {
                if !ignore_nullable && definition.nullable {
                    return false;
                }

                _ = definition;
                true
            }

            // TODO: implement
            fn validate_measurement_definition(
                definition: &MeasurementDefinition,
                ignore_nullable: bool,
            ) -> bool {
                if !ignore_nullable && definition.nullable {
                    return false;
                }

                true
            }

            fn apply_constraints(
                constraints: &<Self::Type as PropertyType>::Constraints,
                value: &Self::Type,
            ) -> Result<(), ConstraintViolationError> {
                let value = value.get::<$unit>();

                let constraints = NumericConstraints {
                    min: constraints.min.map(|x| x.get::<$unit>()),
                    max: constraints.max.map(|x| x.get::<$unit>()),
                };

                f64::apply_constraints(&constraints, &value)
            }

            fn as_constraints(
                constraints: &<Self::Type as PropertyType>::Constraints,
            ) -> Constraints {
                let constraints = NumericConstraints {
                    min: constraints.min.map(|x| x.get::<$unit>()),
                    max: constraints.max.map(|x| x.get::<$unit>()),
                };

                f64::as_constraints(&constraints)
            }
        }
    };
}

with_uom_units!(impl_uom_unit);

// --- option ---
impl<T> PropertyAdapter for Option<T>
where
    T: PropertyAdapter,
    T::Type: PropertyType,
{
    type Type = Option<T::Type>;
    type Input = Option<T::Input>;

    fn convert_input(input: Self::Input) -> Self::Type {
        input.map(T::convert_input)
    }

    fn from_scalar(value: ScalarValue) -> Result<Self::Type, ScalarValueTypeMismatchError> {
        match value {
            ScalarValue::Null => Ok(None),
            value => T::from_scalar(value).map(Some),
        }
    }

    fn into_scalar(value: Self::Type) -> ScalarValue {
        match value {
            Some(value) => T::into_scalar(value),
            None => ScalarValue::Null,
        }
    }

    fn validate_scalar_property_definition(
        definition: &ScalarPropertyDefinition,
        ignore_nullable: bool,
    ) -> bool {
        _ = ignore_nullable;
        T::validate_scalar_property_definition(definition, true)
    }

    fn validate_measurement_definition(
        definition: &MeasurementDefinition,
        ignore_nullable: bool,
    ) -> bool {
        _ = ignore_nullable;
        T::validate_measurement_definition(definition, true)
    }

    fn apply_constraints(
        constraints: &<Self::Type as PropertyType>::Constraints,
        value: &Self::Type,
    ) -> Result<(), ConstraintViolationError> {
        match value {
            Some(value) => T::apply_constraints(constraints, value),
            None => Ok(()),
        }
    }

    fn as_constraints(constraints: &<Self::Type as PropertyType>::Constraints) -> Constraints {
        T::as_constraints(constraints)
    }
}
