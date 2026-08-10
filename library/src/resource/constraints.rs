use std::fmt::Debug;

use regex::Regex;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Unconstrained;

// --- numeric ---
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct NumericConstraints<T> {
    pub(crate) min: Option<T>,
    pub(crate) max: Option<T>,
}

// --- string ---
#[derive(Debug, Clone, Default)]
pub struct StringConstraints {
    pub min_length: Option<usize>,
    pub pattern: Option<(String, Regex)>,
}

impl PartialEq for StringConstraints {
    fn eq(&self, other: &Self) -> bool {
        self.min_length == other.min_length
            && match (&self.pattern, &other.pattern) {
                (None, None) => true,
                (Some((lhs, _)), Some((rhs, _))) => lhs == rhs,
                _ => false,
            }
    }
}

// --- enum ---
#[derive(Debug, Default, Clone, PartialEq)]
pub struct EnumConstraints<T: PartialEq> {
    pub allowed: Vec<T>,
}
