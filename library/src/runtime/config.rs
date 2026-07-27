use std::collections::HashMap;
use std::time::Duration;

use qitech_framework_common::MachineIdentificationUnique;
use qitech_lib::ethercat_hal::MasterConfiguration;

use crate::runtime::types::BuildMachineFn;
use crate::runtime::types::Config;

#[derive(Default)]
pub struct RuntimeConfiguration {
    pub(crate) config: Config,
    pub(crate) machines: Vec<(&'static str, BuildMachineFn)>,
    pub(crate) ethercat_mode: EtherCATMode,
    pub(crate) modbus_rtu_mode: ModbusRtuMode,
}

impl RuntimeConfiguration {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn requests_per_cycle_max(mut self, value: usize) -> Self {
        self.config.requests_per_cycle_max = value;
        self
    }

    pub fn cycle_timeout(mut self, value: Duration) -> Self {
        self.config.cycle_timeout = value;
        self
    }

    pub fn export_interval(mut self, value: Duration) -> Self {
        self.config.export_interval = value;
        self
    }

    pub fn ethercat(mut self, config: EtherCATConfig) -> Self {
        self.ethercat_mode = EtherCATMode::Enabled(config);
        self
    }

    pub fn modbus_rtu_device<S: ToString>(
        mut self,
        id_path: S,
        ident: MachineIdentificationUnique,
    ) -> Self {
        let mut config = match self.modbus_rtu_mode {
            ModbusRtuMode::Enabled(config) => config,
            _ => ModbusRtuConfig {
                bindings: Default::default(),
            },
        };

        config.bindings.insert(id_path.to_string(), ident);
        self.modbus_rtu_mode = ModbusRtuMode::Enabled(config);
        self
    }
}

// --- types ---
#[derive(Default)]
pub enum EtherCATMode {
    #[default]
    Disabled,
    Enabled(EtherCATConfig),

    #[allow(unused)]
    Mock,
}

pub struct EtherCATConfig {
    pub interface_scan_interval: Duration,
    pub master_config: Option<MasterConfiguration>,
    pub stay_in_preop: bool,
}

#[derive(Default)]
pub enum ModbusRtuMode {
    #[default]
    Disabled,
    Enabled(ModbusRtuConfig),

    #[allow(unused)]
    Mock,
}

pub struct ModbusRtuConfig {
    pub bindings: HashMap<String, MachineIdentificationUnique>,
}
