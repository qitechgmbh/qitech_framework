use serde::Deserialize;
use serde::Serialize;

/// Conversion of a unit value into the base unit of its quantity.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Conversion {
    /// The quantity has no base unit; values can only be displayed.
    None,
    /// `base = value * factor + offset`
    Affine { factor: f64, offset: f64 },
    /// `base = reference * 10^(value / multiplier)`
    Log10 { multiplier: f64, reference: f64 },
}

impl Conversion {
    pub const IDENTITY: Self = Self::Affine {
        factor: 1.0,
        offset: 0.0,
    };

    pub fn to_base(&self, value: f64) -> Option<f64> {
        match *self {
            Self::None => None,
            Self::Affine { factor, offset } => Some(value * factor + offset),
            Self::Log10 {
                multiplier,
                reference,
            } => Some(reference * 10f64.powf(value / multiplier)),
        }
    }

    pub fn from_base(&self, value: f64) -> Option<f64> {
        match *self {
            Self::None => None,
            Self::Affine { factor, offset } => Some((value - offset) / factor),
            Self::Log10 {
                multiplier,
                reference,
            } => Some(multiplier * (value / reference).log10()),
        }
    }
}
