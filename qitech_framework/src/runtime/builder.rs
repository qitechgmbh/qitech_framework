use std::any::TypeId;
use std::any::type_name;
use std::time::Duration;

use qitech_framework_core::report::error::BuildError;
use qitech_framework_core::schema::MachineSchema;

use crate::machine::BuildContext;
use crate::machine::Machine;
use crate::machine::MachineBuild;
use crate::machine::MachineDescriptor;
use crate::runtime::error::RuntimeInitializeError;
use crate::runtime::types::BuildMachineFn;
use crate::runtime::types::MachineRegistry;
use crate::runtime::types::MachineRegistryEntry;
use crate::runtime::Runtime;

#[derive(Default)]
pub struct RuntimeBuilder {
    pub(crate) config: RuntimeConfig,
    pub(crate) machines: Vec<MachineRegistration>,
}

impl RuntimeBuilder {
    pub fn build(mut self) -> Result<Runtime<>, RuntimeInitializeError> {
        // --- create machine registry ---
        let mut machine_registry = MachineRegistry::new();

        for MachineRegistration {
            schema,
            build,
            type_id,
            type_name,
        } in self.machines
        {
            let schema = MachineSchema::parse_str(schema)?;
            let ident = schema.identification;

            if machine_registry
                .insert(
                    schema.identification,
                    MachineRegistryEntry {
                        schema: schema.clone(),
                        type_id,
                        type_name,
                        build,
                    },
                )
                .is_some()
            {
                return Err(RuntimeInitializeError::DuplicateMachine(ident));
            }
        }
    }

    pub fn machine<M>(mut self) -> Self
    where
        M: Machine + MachineBuild + MachineDescriptor + 'static,
    {
        fn build_adapter<M>(
            ctx: &mut BuildContext,
        ) -> Result<Box<dyn Machine + 'static>, BuildError>
        where
            M: MachineBuild + Machine + 'static,
        {
            Ok(Box::new(M::build(ctx)?))
        }

        self.machines.push(MachineRegistration {
            schema: M::SCHEMA,
            build: build_adapter::<M>,
            type_id: TypeId::of::<M>(),
            type_name: type_name::<M>(),
        });

        self
    }
}

pub(crate) struct RuntimeConfig {
    pub(crate) requests_per_cycle_max: usize,
    pub(crate) export_interval: Duration,
    pub(crate) cycle_period: Duration,
}

pub(crate) struct MachineRegistration {
    pub(crate) schema: &'static str,
    pub(crate) build: BuildMachineFn,
    pub(crate) type_id: TypeId,
    pub(crate) type_name: &'static str,
}
