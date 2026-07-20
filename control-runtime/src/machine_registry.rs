use std::collections::HashMap;
use control_core::{MachineIdentification, schema::{self, latest::MachineSchema}};
use crate::{Machine, MachineBuild, MachineBuildError, MachineBuildContext};

#[derive(Default)]
pub struct MachineRegistry {
    inner: HashMap<MachineIdentification, MachineRegistryEntry>,
}

impl MachineRegistry {
    pub fn register<T>(&mut self, schema: &'static str) -> anyhow::Result<()>
    where
        T: MachineBuild + Machine + 'static,
    {
        let schema = schema::parse_latest(schema)?;

        let ident = schema.identification;
        let entry = MachineRegistryEntry {
            schema,
            build: Self::build_adapter::<T>,
        };

        if self.inner.insert(ident, entry).is_some() {
            
        };

        Ok(())
    }

    pub(crate) fn find(&self, ident: MachineIdentification) -> Option<&MachineRegistryEntry> {
        self.inner.get(&ident)
    }

    fn build_adapter<T>(
        builder: MachineBuildContext<'_>,
    ) -> Result<Box<dyn Machine>, MachineBuildError>
    where
        T: MachineBuild + Machine + 'static,
    {
        Ok(Box::new(T::build(builder)?))
    }
}

pub struct MachineRegistryEntry {
    #[allow(unused)]
    pub schema: MachineSchema,
    pub build: fn(MachineBuildContext<'_>) -> Result<Box<dyn Machine>, MachineBuildError>,
}
