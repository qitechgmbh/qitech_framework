use std::any::TypeId;
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use qitech_framework_core::ident::MachineInstanceId;
use qitech_framework_core::ident::MachineTypeId;
use qitech_framework_core::report::ResourceKind;
use qitech_framework_core::report::error::BuildError;
use qitech_framework_core::schema::MachineSchema;

use crate::machine::BuildContext;
use crate::machine::BuildResult;
use crate::machine::CommandHandle;
use crate::machine::ConfigPropertyHandle;
use crate::machine::Machine;
use crate::resource::Journals;
use crate::resource::LifetimeTokenOwner;
use crate::resource::PropertyRegistry;
use crate::resource::ResourceRegistry;

pub(crate) type MachineRegistry = HashMap<MachineTypeId, MachineRegistryEntry>;

pub(crate) struct MachineRegistryEntry {
    pub(crate) schema: MachineSchema,
    pub(crate) type_id: TypeId,
    pub(crate) type_name: &'static str,
}

pub(crate) struct MachineInstance {
    pub(crate) ident: MachineInstanceId,
    pub(crate) machine: Box<dyn Machine>,
    pub(crate) configs: HashMap<&'static str, ConfigPropertyHandle>,
    pub(crate) commands: HashMap<&'static str, CommandHandle>,
    pub(crate) subscriptions: HashMap<MachineInstanceId, LifetimeTokenOwner>,
}

pub struct Runtime2 {
    runtime_id: u64,

    // --- resource managment ---
    journals: Journals,
    resources: ResourceRegistry,

    // --- instances ---
    machine_registry: MachineRegistry,
    machine_instances: Vec<MachineInstance>,
}

impl Runtime2 {
    pub fn new() -> Self {
        static RUNTIME_ID: AtomicU64 = AtomicU64::new(0);
        let runtime_id = RUNTIME_ID.fetch_add(1, Ordering::Relaxed);

        Self {
            runtime_id,
            resources: ResourceRegistry {
                config_properties: PropertyRegistry::new(ResourceKind::ConfigProperty, 4096),
                state_properties: PropertyRegistry::new(ResourceKind::StateProperty, 4096),
                measurements: PropertyRegistry::new(ResourceKind::Measurement, 4096),
            },
            journals: Journals::default(),
            machine_instances: Default::default(),
            machine_registry: Default::default(),
        }
    }

    pub fn machine<B>(mut self, instance_id: u16, mut builder: B) -> Result<(), ()>
    where
        B: MachineBuilder + 'static,
    {
        fn build_adapter<B>(
            ctx: &mut BuildContext,
        ) -> Result<Box<dyn Machine + 'static>, BuildError>
        where
            B: MachineBuilder + 'static,
        {
            Ok(Box::new(B::build(ctx)?))
        }

        for slot in builder.slots() {
            
        }

        /*
        self.machines.push(MachineRegistration {
            schema: M::SCHEMA,
            build: build_adapter::<M>,
            type_id: TypeId::of::<M>(),
            type_name: type_name::<M>(),
        });
        */

        Ok(MachineHandle {
            runtime_id: self.runtime_id,
            instance_id,
        })
    }
}

pub trait MachineBuilder {
    type Output: Machine;
    fn build(ctx: BuildContext) -> BuildResult<Self::Output>;
    fn slots(&mut self) -> Vec<&mut dyn HardwareSlot>;
}

pub(crate) trait HardwareSlot {
    fn stamp(&mut self, runtime_id: u64);
}
