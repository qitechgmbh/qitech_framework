//! Unit catalog read from `units.kdl`.
//!
//! Top-level nodes are quantities, their children are units:
//!
//! ```kdl
//! length {
//!     meter      symbol="m"  base=#true
//!     millimeter symbol="mm" factor=1e-3
//! }
//!
//! sound_pressure {
//!     pascal symbol="Pa" base=#true
//!     db_spl symbol="dB" log10_multiplier=20 log10_reference=2e-5
//! }
//! ```
//!
//! A unit is referenced as `quantity.unit`, e.g. `length.millimeter`.

mod catalog;
mod conversion;
mod error;
mod parser;

#[cfg(test)]
mod tests;

pub use catalog::Quantity;
pub use catalog::Unit;
pub use catalog::UnitCatalog;
pub use conversion::Conversion;
pub use error::Issue;
pub use error::UnitCatalogError;
