use std::any::TypeId;
use std::any::type_name;
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;

use qitech_framework_core::report::error::BuildError;
use qitech_framework_core::report::ResourceKind;
use qitech_framework_core::schema::MachineSchema;
use serialport::DataBits;
use serialport::Parity;
use serialport::StopBits;
use tokio_modbus::SlaveId;

use crate::machine::BuildContext;
use crate::machine::Machine;
use crate::machine::MachineBuild;
use crate::machine::MachineDescriptor;
use crate::resource::PropertyRegistry;
use crate::resource::ResourceRegistry;
use crate::runtime::error::RuntimeInitializeError;
use crate::runtime::types::BuildMachineFn;
use crate::runtime::types::MachineRegistry;
use crate::runtime::types::MachineRegistryEntry;
use crate::runtime::Runtime;

#[derive(Default)]
pub struct RuntimeBuilder {
    pub(crate) config: RuntimeConfig,
    pub(crate) machines: Vec<MachineRegistration>,
    pub(crate) modbus_rtu_buses: Vec<(ModbusRtuPort, ModbusRTUBusBuilder)>, 
}

impl RuntimeBuilder {
    pub fn new() -> Self {

    }

    pub fn build(mut self) -> Result<Runtime, RuntimeInitializeError> {
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

        Ok(Runtime { 
            report: Default::default(), 
            journals: Default::default(),
            resources: ResourceRegistry {
                config_properties: PropertyRegistry::new(ResourceKind::ConfigProperty, 4096),
                state_properties: PropertyRegistry::new(ResourceKind::StateProperty, 4096),
                measurements: PropertyRegistry::new(ResourceKind::Measurement, 4096),
            }, 
            machine_registry, 
            machine_instances: Default::default(), 
            config: self.config, 
            last_export_ts: Instant::now(), 
            export_count: Rc::new(Cell::new(0)),
        })
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

    pub fn modbus_rtu<F>(mut self, port: ModbusRtuPort, build: F) -> Self
    where
        F: Fn(ModbusRTUBusBuilder) -> ModbusRTUBusBuilder
    {
        self.modbus_rtu_buses.push((port, build(ModbusRTUBusBuilder::new())));
    }

    // -> accept a controller or master ?
    // pub fn ethercat(mut self, machine: impl EtherCATMachine, instance_id: u16) -> Self {
    //     
    // }
}

fn test() {
    let machine = runtime_builder;

    let laser_0 = idk.machine::<LaserV1>(10)?;

    let laser_0_bus = idk.modbus_rtu.modbus_rtu(
            ModbusRtuPort::topology("pci-0000:c6:00.0-usbv2-0:2.2:1.0-port0"), 
            |builder| builder
                .baud_rate(9600)
                .data_bits(DataBits::Eight)
                .stop_bits(StopBits::One)
                .parity(Parity::None)
                .device(1, laser_0, 0)
        );

    let builder = RuntimeBuilder::new()
        .modbus_rtu(
            ModbusRtuPort::topology("pci-0000:c6:00.0-usbv2-0:2.2:1.0-port0"), 
            |builder| builder
                .baud_rate(9600)
                .data_bits(DataBits::Eight)
                .stop_bits(StopBits::One)
                .parity(Parity::None)
                .device(1, laser_0, 0)
        )
        .xtrem(
            XtremPort::udp(""),
            |builder| builder
                .device(1, 1, 1)
        );
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

pub struct ModbusRTUBusBuilder {
    baud_rate: u32,
    data_bits: DataBits,
    parity: Parity,
    stop_bits: StopBits,
    devices: HashMap<u8, (u16, u16)>
}

impl ModbusRTUBusBuilder {
    pub(crate) fn new() -> Self {
        Self {
            baud_rate: 9600,
            data_bits: 8,
            parity: Parity::None,
            stop_bits: 1,
            devices: Default::default()
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

pub trait MachineHardware {

}

#[derive(Debug, Hash)]
pub enum ModbusRtuPort { 
    Topology(String),
    Device(String),
}

impl ModbusRtuPort { 
    /// Creates a Topology variant from any type that can be converted into a String.
    pub fn topology(val: impl Into<String>) -> Self {
        Self::Topology(val.into())
    }

    /// Creates a Device variant from any type that can be converted into a String.
    pub fn device(val: impl Into<String>) -> Self {
        Self::Device(val.into())
    }
}
