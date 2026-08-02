use qitech_framework_core::report::MachineConfigPropertyConstraints;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct NumericConfigPropertyConstraints<T: Copy + PartialOrd> {
    min: Option<T>,
    max: Option<T>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct StringConfigPropertyConstraints {
    min_length: Option<usize>,
    max_length: Option<usize>,
    pattern: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct EnumConfigPropertyConstraints<T: PartialEq + ToString> {
    allowed: Vec<T>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct BooleanConfigPropertyConstraints;

impl Into<MachineConfigPropertyConstraints> for BooleanConfigPropertyConstraints {
    fn into(self) -> MachineConfigPropertyConstraints {
        MachineConfigPropertyConstraints::None
    }
}

impl Into<MachineConfigPropertyConstraints> for StringConfigPropertyConstraints {
    fn into(self) -> MachineConfigPropertyConstraints {
        MachineConfigPropertyConstraints::String {
            min_length: self.min_length,
            max_length: self.max_length,
            pattern: self.pattern,
        }
    }
}

impl<T: PartialEq + ToString> Into<MachineConfigPropertyConstraints>
    for EnumConfigPropertyConstraints<T>
{
    fn into(self) -> MachineConfigPropertyConstraints {
        MachineConfigPropertyConstraints::Enum {
            allowed: self
                .allowed
                .into_iter()
                .map(|item| item.to_string())
                .collect(),
        }
    }
}

impl Into<MachineConfigPropertyConstraints> for NumericConfigPropertyConstraints<i64> {
    fn into(self) -> MachineConfigPropertyConstraints {
        MachineConfigPropertyConstraints::Integer {
            min: self.min,
            max: self.max,
        }
    }
}

impl Into<MachineConfigPropertyConstraints> for NumericConfigPropertyConstraints<f64> {
    fn into(self) -> MachineConfigPropertyConstraints {
        MachineConfigPropertyConstraints::Float {
            min: self.min,
            max: self.max,
        }
    }
}

// Type millimeter -> user provides millimeter as input NOT Length
