use std::fmt::Debug;

use qitech_framework_core::report::ConstraintViolationError;
use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub struct NumericConstraints<T: Copy + PartialOrd + PartialEq> {
    min: Option<T>,
    max: Option<T>,
}

impl<T: Copy + PartialOrd + PartialEq> NumericConstraints<T> {
    pub fn new(min: Option<T>, max: Option<T>) -> Result<Self, ConstraintViolationError> {
        Self::validate(min, max)?;
        Ok(Self { min, max })
    }

    pub fn set_min(&mut self, min: Option<T>) -> Result<bool, ConstraintViolationError> {
        Self::validate(min, self.max)?;

        if self.min == min {
            return Ok(false);
        }

        self.min = min;
        Ok(true)
    }

    pub fn set_max(&mut self, max: Option<T>) -> Result<bool, ConstraintViolationError> {
        Self::validate(self.min, max)?;

        if self.max == max {
            return Ok(false);
        }

        self.max = max;
        Ok(true)
    }

    pub fn min(&self) -> Option<T> {
        self.min
    }

    pub fn max(&self) -> Option<T> {
        self.max
    }

    fn validate(min: Option<T>, max: Option<T>) -> Result<(), ConstraintViolationError> {
        if let (Some(min), Some(max)) = (min, max)
            && min > max
        {
            return Err(ConstraintViolationError::InvalidRange {
                min: min.into(),
                max: max.into(),
            });
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
impl<T: Copy + PartialOrd + PartialEq> Default for NumericConstraints<T> {
    fn default() -> Self {
        Self {
            min: None,
            max: None,
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
