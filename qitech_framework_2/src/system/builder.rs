use std::any::TypeId;
use std::any::type_name;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use qitech_framework_core::schema::MachineSchema;
use qitech_framework_core::system::SystemSchema;
use qitech_framework_core::units::UnitCatalog;

use crate::descriptor::MachineDescriptor;
use crate::system::MachineType;
use crate::system::System;
use crate::system::SystemLoadError;
use crate::system::resolve;

// --- builder ---
/// Registers the unit catalogs and machine types a system file may `import`.
///
/// Registration problems are collected and reported by `load`, so the chain never breaks.
#[derive(Debug, Default)]
pub struct SystemBuilder {
    units: BTreeMap<String, UnitCatalog>,
    machine_types: BTreeMap<String, MachineType>,
    issues: Vec<String>,
}

impl SystemBuilder {
    /// Registers a unit catalog, e.g. `si`, importable by machine schemas and the system file.
    pub fn units(mut self, name: impl Into<String>, catalog: UnitCatalog) -> Self {
        let name = name.into();
        if self.is_registered(&name) {
            self.issues.push(format!("`{name}` is registered twice"));
        } else {
            self.units.insert(name, catalog);
        }
        self
    }

    /// Registers a machine type under the `name` of its embedded schema.
    pub fn machine<M: MachineDescriptor + 'static>(mut self) -> Self {
        let type_name = type_name::<M>();

        let schema = match MachineSchema::parse_str(M::SCHEMA) {
            Ok(schema) => schema,
            Err(e) => {
                self.issues
                    .push(format!("schema of `{type_name}` is invalid:\n{e}"));
                return self;
            }
        };

        let name = schema.name.clone();
        if self.is_registered(&name) {
            self.issues
                .push(format!("`{name}` (from `{type_name}`) is registered twice"));
            return self;
        }

        self.machine_types.insert(
            name,
            MachineType {
                schema,
                type_id: TypeId::of::<M>(),
                type_name,
            },
        );
        self
    }

    pub fn load(self, path: impl AsRef<Path>) -> Result<System, SystemLoadError> {
        let path = path.as_ref();
        let input = fs::read_to_string(path).map_err(|source| SystemLoadError::Read {
            path: path.to_owned(),
            source,
        })?;
        self.load_str(&input)
    }

    pub fn load_str(self, input: &str) -> Result<System, SystemLoadError> {
        let schema = SystemSchema::parse_str(input)?;

        let mut issues = self.issues;
        resolve::resolve(&schema, &self.units, &self.machine_types, &mut issues);
        if !issues.is_empty() {
            return Err(SystemLoadError::Resolve(issues));
        }

        Ok(System {
            schema,
            units: self.units,
            machine_types: self.machine_types,
        })
    }

    /// Units and machine types share one namespace, since both are named by `import`.
    fn is_registered(&self, name: &str) -> bool {
        self.units.contains_key(name) || self.machine_types.contains_key(name)
    }
}
