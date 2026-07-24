
use std::collections::HashMap;
use qitech_framework_common::{MachineIdentification, MachineSchema};
use crate::machine::{BuildContext, Machine, MachineBuild, MachineInterface, error::BuildResult};

#[derive(Default)]
pub struct MachineRegistry {
    inner: HashMap<MachineIdentification, MachineRegistryEntry>,
}

impl MachineRegistry {
    pub fn register<T>(&mut self) -> anyhow::Result<()>
    where
        T: Machine + MachineInterface + MachineBuild + 'static,
    {
        let schema = MachineSchema::from_yaml_str(T::SCHEMA)?;
        
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
        builder: BuildContext<'_>,
    ) -> BuildResult<Box<dyn Machine>>
    where
        T: MachineBuild + Machine + 'static,
    {
        Ok(Box::new(T::build(builder)?))
    }
}

pub struct MachineRegistryEntry {
    #[allow(unused)]
    pub schema: MachineSchema,
    pub build: fn(BuildContext<'_>) -> BuildResult<Box<dyn Machine>>,
}
