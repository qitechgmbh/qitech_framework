use std::time::Duration;
use qitech_framework_common::MachineSchema;
use qitech_lib::ethercat_hal::MasterConfiguration;

use crate::machine::BuildContext;
use crate::machine::Machine;
use crate::machine::MachineBuild;
use crate::machine::MachineInterface;
use crate::machine::Resources;
use crate::machine::error::BuildResult;
use crate::runtime::MachineRegistry;
use crate::runtime::Runtime;
use crate::runtime::init::error::RuntimeInitializeError;
use crate::runtime::init::error::RuntimeInitializeResult;
use crate::runtime::types::BuildMachineFn;

mod error;
mod ethercat;
mod hub;

pub struct RuntimeInitializer {
    machines: Vec<(&'static str, BuildMachineFn)>,
    ethercat_mode: EtherCATMode,
    modbus_rtu_mode: ModbusRtuMode,
    modbus_tcp_mode: ModbusTcpMode,
}

impl RuntimeInitializer {
    pub(crate) fn new() -> Self {
        Self { 
            machines: Default::default(), 
            ethercat_mode: EtherCATMode::Disabled, 
            modbus_rtu_mode: ModbusRtuMode::Disabled, 
            modbus_tcp_mode: ModbusTcpMode::Disabled,
        }
    }

    pub fn with_ethercat(mut self, config: EtherCATConfig) -> Self {
        self.ethercat_mode = EtherCATMode::Enabled(config);
        self
    }

    // pub fn with_ethercat_mock(mut self) -> Self {
    //     self.ethercat_mode = EtherCATMode::Mock;
    //     self
    // }

    pub fn with_modbus_rtu(mut self) -> Self {
        self.modbus_rtu_mode = ModbusRtuMode::Enabled;
        self
    }

    pub fn with_machine<M>(mut self) -> Self
    where 
        M: Machine + MachineBuild + MachineInterface + 'static,
    {
        self.machines.push((M::SCHEMA, build_adapter::<M>));
        self
    }

    /// attempts to create a new runtime with the provided configuration
    pub fn build(self) -> RuntimeInitializeResult<Runtime> {
        // --- step one: connect to hub ---

        // --- step two: crate machine registry ---
        let mut machine_registry = MachineRegistry::default();

        for (schema_str, build_fn) in self.machines {
            let schema = MachineSchema::from_yaml_str(schema_str)?;

            if machine_registry.insert(schema.identification, build_fn).is_some() {
                return Err(RuntimeInitializeError::DuplicateMachine(schema.identification));
            }
        }

        // --- step three: initialize hardware ---
        let mut hardware_registry = Default::default();

        let (ecat_controller, sub_devices) = if let EtherCATMode::Enabled(config) = self.ethercat_mode {
            let (controller, sub_devices) = ethercat::init(config, &mut hardware_registry)?;
            (Some(controller), sub_devices)
        } else {
            (None, Default::default())
        };

        // --- finish ---
        Ok(Runtime {
            machine_registry,
            hardware_registry,
            resources: Resources::default(),
            ecat_controller,
            machines: Default::default(),
            sub_devices,
        })
    }
}

// --- types ---
pub enum EtherCATMode {
    Disabled,
    Enabled(EtherCATConfig),
    Mock,
}

pub struct EtherCATConfig {
    pub interface_discovery_retry_interval: Duration,
    pub master_config: Option<MasterConfiguration>,
}

pub enum ModbusRtuMode {
    Disabled,
    Enabled,
    Mock,
}

pub enum ModbusTcpMode {
    Disabled,
    Enabled,
    Mock,
}

// --- errors ---
pub enum InitializeRuntimeError {
    HubUnreachable,
}

// --- utils ---
fn build_adapter<T>(
    builder: BuildContext<'_>,
) -> BuildResult<Box<dyn Machine>>
where
    T: MachineBuild + Machine + 'static,
{
    Ok(Box::new(T::build(builder)?))
}