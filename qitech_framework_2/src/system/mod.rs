//! A system file wires machine instances to buses; Rust registers what its `import`s may name.
//!
//! ```ignore
//! let system = System::builder()
//!     .units("si", UnitCatalog::parse_str(SI)?)
//!     .machine::<WinderV1>()
//!     .machine::<LaserV1>()
//!     .load("system.kdl")?;
//! ```

mod builder;
mod error;
mod resolve;

#[cfg(test)]
mod tests;

use std::any::TypeId;
use std::collections::BTreeMap;

pub use builder::SystemBuilder;
pub use error::SystemLoadError;
use qitech_framework_core::schema::MachineSchema;
use qitech_framework_core::system::SystemSchema;
use qitech_framework_core::units::UnitCatalog;

// --- system ---
/// A parsed system file whose imports, machine types and unit references are all resolved.
#[derive(Debug)]
pub struct System {
    schema: SystemSchema,
    units: BTreeMap<String, UnitCatalog>,
    machine_types: BTreeMap<String, MachineType>,
}

impl System {
    pub fn builder() -> SystemBuilder {
        SystemBuilder::default()
    }

    pub fn schema(&self) -> &SystemSchema {
        &self.schema
    }

    pub fn units(&self, name: &str) -> Option<&UnitCatalog> {
        self.units.get(name)
    }

    pub fn machine_type(&self, name: &str) -> Option<&MachineType> {
        self.machine_types.get(name)
    }

    /// Machine instances with their resolved type, ordered by instance name.
    pub fn machines(&self) -> impl Iterator<Item = (&str, &MachineType)> {
        self.schema.machines.iter().map(|(instance, ty)| {
            let machine_type = &self.machine_types[ty];
            (instance.as_str(), machine_type)
        })
    }
}

// --- machine type ---
/// A Rust machine type registered under the `name` of its schema.
#[derive(Debug)]
pub struct MachineType {
    pub schema: MachineSchema,
    pub type_id: TypeId,
    pub type_name: &'static str,
}
