use kdl::KdlDocument;
use serde::Deserialize;
use serde::Serialize;

use crate::units::Conversion;
use crate::units::UnitCatalogError;
use crate::units::error::Issues;
use crate::units::parser;

// --- catalog ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitCatalog {
    pub quantities: Vec<Quantity>,
}

impl UnitCatalog {
    pub fn parse_str(input: &str) -> Result<Self, UnitCatalogError> {
        let document: KdlDocument = input
            .parse()
            .map_err(|e: kdl::KdlError| UnitCatalogError::from_kdl(input, e))?;

        let mut issues = Issues::new(input);
        let catalog = parser::parse_catalog(&document, &mut issues);
        issues.finish(catalog)
    }

    pub fn quantity(&self, name: &str) -> Option<&Quantity> {
        self.quantities.iter().find(|q| q.name == name)
    }

    /// Looks up a unit by its reference, e.g. `length.millimeter`.
    pub fn unit(&self, reference: &str) -> Option<(&Quantity, &Unit)> {
        let (quantity, unit) = reference.split_once('.')?;
        let quantity = self.quantity(quantity)?;
        Some((quantity, quantity.unit(unit)?))
    }
}

// --- quantity ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quantity {
    pub name: String,
    pub units: Vec<Unit>,
}

impl Quantity {
    pub fn unit(&self, name: &str) -> Option<&Unit> {
        self.units.iter().find(|u| u.name == name)
    }

    /// The base unit. `None` means units of this quantity are not convertible.
    pub fn base(&self) -> Option<&Unit> {
        self.units.iter().find(|u| u.base)
    }

    pub fn is_convertible(&self) -> bool {
        self.base().is_some()
    }
}

// --- unit ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    pub name: String,
    pub symbol: String,
    pub base: bool,
    pub conversion: Conversion,
}
