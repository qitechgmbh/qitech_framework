use std::collections::HashMap;
use control_core::{MachineIdentification, schema::{self, v1_0::MachineSchema}};
use crate::{Machine, MachineBuild, MachineBuildError, BuildContext};

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

        self.inner.insert(schema.identification, MachineRegistryEntry {
            schema,
            build: Self::build_adapter::<T>,
        });

        Ok(())
    }

    pub(crate) fn find(&self, ident: MachineIdentification) -> Option<&MachineRegistryEntry> {
        self.inner.get(&ident)
    }

    fn build_adapter<T>(
        builder: BuildContext<'_>,
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
    pub build: fn(BuildContext<'_>) -> Result<Box<dyn Machine>, MachineBuildError>,
}
