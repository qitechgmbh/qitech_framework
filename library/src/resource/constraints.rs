use std::fmt::Debug;

use qitech_framework_core::NumericValue;
use qitech_framework_core::report::ConstraintViolationError;
use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub struct NumericConstraints {
    min: NumericValue,
    max: NumericValue,
}

impl NumericConstraints {
    pub fn new(
        min: NumericValue,
        max: NumericValue,
    ) -> Result<Self, ConstraintViolationError> {
        Self::validate(&min, &max)?;
        Ok(Self { min, max })
    }

    pub fn set_min(
        &mut self,
        min: NumericValue,
    ) -> Result<bool, ConstraintViolationError> {
        Self::validate(&min, &self.max)?;

        if self.min == min {
            return Ok(false);
        }

        self.min = min;
        Ok(true)
    }

    pub fn set_max(
        &mut self,
        max: NumericValue,
    ) -> Result<bool, ConstraintViolationError> {
        Self::validate(&self.min, &max)?;

        if self.max == max {
            return Ok(false);
        }

        self.max = max;
        Ok(true)
    }

    pub fn min(&self) -> &NumericValue {
        &self.min
    }

    pub fn max(&self) -> &NumericValue {
        &self.max
    }

    fn validate(
        min: &NumericValue,
        max: &NumericValue,
    ) -> Result<(), ConstraintViolationError> {
        match (min, max) {
            (NumericValue::Integer(Some(min)), NumericValue::Integer(Some(max))) => {
                if min > max {
                    return Err(ConstraintViolationError::InvalidRange {
                        min: NumericValue::Integer(Some(*min)),
                        max: NumericValue::Integer(Some(*max)),
                    });
                }
            }

            (NumericValue::Float(Some(min)), NumericValue::Float(Some(max))) => {
                if min > max {
                    return Err(ConstraintViolationError::InvalidRange {
                        min: NumericValue::Float(Some(*min)),
                        max: NumericValue::Float(Some(*max)),
                    });
                }
            }

            (NumericValue::Integer(_), NumericValue::Float(_))
            | (NumericValue::Float(_), NumericValue::Integer(_)) => {
                panic!("Numeric constraint min and max must have the same type");
            }

            _ => {}
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OptionalNumericConstraints<T> {
    pub(crate) min: Option<T>,
    pub(crate) max: Option<T>,
    pub(crate) allow_none: bool,
}

#[derive(Debug, Clone)]
pub struct OptionalStringConstraints {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<(&'static str, Regex)>,
    pub allow_none: bool,
}

impl PartialEq for OptionalStringConstraints {
    fn eq(&self, other: &Self) -> bool {
        self.allow_none == other.allow_none
            && self.min_length == other.min_length
            && self.max_length == other.max_length
            && match (&self.pattern, &other.pattern) {
                (None, None) => true,
                (Some((lhs, _)), Some((rhs, _))) => lhs == rhs,
                _ => false,
            }
    }
}

// --- string ---
#[derive(Debug, Clone, Default)]
pub struct StringConstraints {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<(&'static str, Regex)>,
}

impl PartialEq for StringConstraints {
    fn eq(&self, other: &Self) -> bool {
        self.min_length == other.min_length
            && self.max_length == other.max_length
            && match (&self.pattern, &other.pattern) {
                (None, None) => true,
                (Some((lhs, _)), Some((rhs, _))) => lhs == rhs,
                _ => false,
            }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumConstraints<T: PartialEq> {
    pub allowed: Vec<T>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OptionalEnumConstraints<T: PartialEq> {
    pub allowed: Vec<T>,
    pub allow_none: bool,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Unconstrained;

// --- default ---
impl Default for NumericConstraints {
    fn default() -> Self {
        Self {
            min: NumericValue::Float(None),
            max: NumericValue::Float(None),
        }
    }
}

impl<T> Default for OptionalNumericConstraints<T> {
    fn default() -> Self {
        Self {
            min: None,
            max: None,
            allow_none: true,
        }
    }
}

impl Default for OptionalStringConstraints {
    fn default() -> Self {
        Self {
            min_length: None,
            max_length: None,
            pattern: None,
            allow_none: true,
        }
    }
}

impl<T: PartialEq> Default for EnumConstraints<T> {
    fn default() -> Self {
        Self {
            allowed: Vec::new(),
        }
    }
}

impl<T: PartialEq> Default for OptionalEnumConstraints<T> {
    fn default() -> Self {
        Self {
            allowed: Vec::new(),
            allow_none: true,
        }
    }
}
