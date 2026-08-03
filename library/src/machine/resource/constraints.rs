#[derive(Debug, Default, Clone, PartialEq)]
pub struct NumericConfigPropertyConstraints<T: Copy + PartialOrd> {
    pub min: Option<T>,
    pub max: Option<T>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct StringConfigPropertyConstraints {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct EnumConfigPropertyConstraints<T: PartialEq + ToString> {
    pub allowed: Vec<T>,
}
