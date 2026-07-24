use std::collections::HashMap;
use std::time::Duration;

use qitech_framework_common::MachineIdentification;
use qitech_lib::ethercat_hal::MasterConfiguration;

use crate::runtime::MachineRegistryEntry;
use crate::runtime::Runtime;
use crate::runtime::init::error::RuntimeInitializeResult;

mod error;
mod ethercat;
mod hub;

pub struct RuntimeInitializer {
    machine_registry: HashMap<MachineIdentification, MachineRegistryEntry>,
    ethercat_mode: EtherCATMode,
    modbus_rtu_mode: ModbusRtuMode,
    modbus_tcp_mode: ModbusTcpMode,
}

impl RuntimeInitializer {
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

    /// attempts to create a new runtime with the provided configuration
    pub fn create(mut self) -> RuntimeInitializeResult<Runtime> {
        // TODO: connect to hub

        let mut hardware_registry = Default::default();

        let (controller, sub_devices) = if let EtherCATMode::Enabled(config) = self.ethercat_mode {
            let (controller, sub_devices) = ethercat::init(config, &mut hardware_registry)?;
            (Some(controller), sub_devices)
        } else {
            (None, Default::default())
        };

        Ok(Runtime {
            machine_registry: self.machine_registry,
            hardware_registry,
            config_properties: todo!(),
            state_properties: todo!(),
            measurements: todo!(),
            commands: todo!(),
            events: todo!(),
            ecat_controller: todo!(),
            machines: todo!(),
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
    pub master: Option<MasterConfiguration>,
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
