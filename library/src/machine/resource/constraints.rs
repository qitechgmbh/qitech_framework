use qitech_framework_core::report::MachineConfigPropertyConstraints;

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
