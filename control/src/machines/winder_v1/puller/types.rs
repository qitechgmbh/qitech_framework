#[derive(Debug, Clone, Default)]
pub enum SpeedRegulationMode {
    #[default]
    Speed,
    AdaptiveDiameter,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GearRatio {
    OneToOne,
    OneToFive,
    OneToTen,
}

impl GearRatio {
    /// Get the speed multiplier for this gear ratio
    pub fn multiplier(&self) -> f64 {
        match self {
            Self::OneToOne => 1.0,
            Self::OneToFive => 5.0,
            Self::OneToTen => 10.0,
        }
    }
}

impl Default for GearRatio {
    fn default() -> Self {
        GearRatio::OneToOne
    }
}
