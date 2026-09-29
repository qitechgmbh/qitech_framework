use std::any::TypeId;
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use qitech_framework_core::ident::MachineIdentification;
use qitech_framework_core::ident::MachineInstanceId;
use qitech_framework_core::report::ResourceKind;
use qitech_framework_core::report::error::BuildError;
use qitech_framework_core::schema::MachineSchema;
use serialport::DataBits;
use serialport::Parity;
use serialport::StopBits;
use tokio_modbus::SlaveId;

use crate::machine::BuildContext;
use crate::machine::CommandHandle;
use crate::machine::ConfigPropertyHandle;
use crate::machine::Machine;
use crate::machine::MachineBuild;
use crate::machine::MachineDescriptor;
use crate::resource::Journals;
use crate::resource::LifetimeTokenOwner;
use crate::resource::PropertyRegistry;
use crate::resource::ResourceRegistry;

pub(crate) type MachineRegistry = HashMap<MachineIdentification, MachineRegistryEntry>;

pub(crate) type BuildMachineFn =
    fn(&mut BuildContext) -> Result<Box<dyn Machine + 'static>, BuildError>;

pub(crate) struct MachineRegistryEntry {
    pub(crate) schema: MachineSchema,
    pub(crate) type_id: TypeId,
    pub(crate) type_name: &'static str,
    pub(crate) build: BuildMachineFn,
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
    modbus_rtu_buses: Vec<ModbusRTUBusConfig>,

    // --- resource managment ---
    journals: Journals,
    resources: ResourceRegistry,

    // --- instances ---
    machines: HashMap<u16, ()>,
    machine_registry: MachineRegistry,
    machine_instances: Vec<MachineInstance>,
}

impl Runtime2 {
    pub fn new() -> Self {
        static RUNTIME_ID: AtomicU64 = AtomicU64::new(0);
        let runtime_id = RUNTIME_ID.fetch_add(1, Ordering::Relaxed);

        Self {
            runtime_id,
            machines: Default::default(),
            resources: ResourceRegistry {
                config_properties: PropertyRegistry::new(ResourceKind::ConfigProperty, 4096),
                state_properties: PropertyRegistry::new(ResourceKind::StateProperty, 4096),
                measurements: PropertyRegistry::new(ResourceKind::Measurement, 4096),
            },
            journals: Journals::default(),
            modbus_rtu_buses: Default::default(),
            machine_instances: Default::default(),
            machine_registry: Default::default(),
        }
    }

    pub fn register_machine<M>(mut self, instance_id: u16) -> Result<MachineHandle, ()>
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

    pub fn register_modbus_rtu_bus<F>(&mut self, config: ModbusRTUBusConfig) -> Result<(), String> {
        self.modbus_rtu_buses.push(config);
        Ok(())
    }
}

pub struct MachineHandle {
    runtime_id: u16,
    instance_id: u16,
}

pub struct ModbusRTUBusConfig {
    port: ModbusRtuPort,
    baud_rate: u32,
    data_bits: DataBits,
    parity: Parity,
    stop_bits: StopBits,
    devices: HashMap<u8, (MachineHandle, u16)>,
}

impl ModbusRTUBusConfig {
    pub(crate) fn new(port: ModbusRtuPort) -> Self {
        Self {
            port,
            baud_rate: 9600,
            data_bits: 8,
            parity: Parity::None,
            stop_bits: 1,
            devices: Default::default(),
        }
    }

    pub fn baud_rate(mut self, baud_rate: u32) -> Self {
        self.baud_rate = baud_rate;
        self
    }

    pub fn data_bits(mut self, data_bits: u8) -> Self {
        self.data_bits = data_bits;
        self
    }

    pub fn parity(mut self, parity: Parity) -> Self {
        self.parity = parity;
        self
    }

    pub fn stop_bits(mut self, stop_bits: u8) -> Self {
        self.stop_bits = stop_bits;
        self
    }

    pub fn device(mut self, slave_id: SlaveId, instance_id: u16, hardware_id: u16) -> Self {
        self.devices.insert(slave_id, (instance_id, hardware_id));
        self
    }
}

#[derive(Debug, Hash)]
pub enum ModbusRtuPort {
    Topology(String),
    Path(String),
}

impl ModbusRtuPort {
    pub fn topology(val: impl Into<String>) -> Self {
        Self::Topology(val.into())
    }

    pub fn path(val: impl Into<String>) -> Self {
        Self::Path(val.into())
    }
}

pub struct ModbusDevice {}

impl ModbusDevice {}
