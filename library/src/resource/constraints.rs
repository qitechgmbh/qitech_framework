use regex::Regex;

#[derive(Clone, PartialEq)]
pub struct NumericConstraints<T: Copy + PartialOrd + PartialEq> {
    pub min: Option<T>,
    pub max: Option<T>,
}

#[derive(Clone, PartialEq)]
pub struct OptionalNumericConstraints<T> {
    pub min: Option<T>,
    pub max: Option<T>,
    pub allow_none: bool,
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

// ---
#[derive(Debug, Clone)]
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

#[derive(Clone, PartialEq)]
pub struct EnumConstraints<T: PartialEq> {
    pub allowed: Vec<T>,
}

#[derive(Clone, PartialEq)]
pub struct OptionalEnumConstraints<T: PartialEq> {
    pub allowed: Vec<T>,
    pub allow_none: bool,
}

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

impl Default for StringConstraints {
    fn default() -> Self {
        Self {
            min_length: None,
            max_length: None,
            pattern: None,
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