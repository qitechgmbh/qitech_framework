use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_lib::ethercat_hal::MasterConfiguration;
use qitech_lib::modbus::ModbusDevice;
use qitech_lib::modbus::ModbusSettings;

use crate::machine::BuildContext;
use crate::machine::Machine;
use crate::machine::MachineBuild;
use crate::machine::MachineInterface;
use crate::machine::error::BuildResult;
use crate::runtime::types::BuildMachineFn;
use crate::runtime::types::Config;
use crate::runtime::types::MachineInstance;

#[derive(Default)]
pub struct RuntimeConfiguration {
    pub(crate) config: Config,
    pub(crate) machines: Vec<(&'static str, BuildMachineFn, TypeId)>,
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

    pub fn modbus_rtu_device<D: ModbusDevice + 'static>(
        mut self,
        id_path: impl ToString,
        ident: MachineIdentificationUnique,
        slave_id: u8,
        settings: Option<ModbusSettings>,
    ) -> Self {
        let mut config = match self.modbus_rtu_mode {
            ModbusRtuMode::Enabled(config) => config,
            _ => ModbusRtuConfig {
                entries: Default::default(),
            },
        };

        let init = Box::new(move |path: String| {
            let dev = D::new(path, slave_id, settings).map_err(|e| format!("{e}"))?;
            let dev: Rc<RefCell<dyn ModbusDevice>> = Rc::new(RefCell::new(dev));
            Ok(dev)
        });

        config
            .entries
            .insert(id_path.to_string(), ModbusRtuEntry { ident, init });

        self.modbus_rtu_mode = ModbusRtuMode::Enabled(config);
        self
    }

    pub fn machine<M>(mut self) -> Self
    where
        M: Machine + MachineBuild + MachineInterface + 'static,
    {
        fn build_adapter<M>(mut ctx: BuildContext) -> BuildResult<MachineInstance>
        where
            M: MachineBuild + Machine + 'static,
        {
            let ident = ctx.ident;
            let machine = Box::new(M::build(&mut ctx)?);

            // --- commit all reserved resources to make them usable ---
            ctx.config_properties.commit();
            ctx.state_properties.commit();
            ctx.measurements.commit();

            Ok(MachineInstance { ident, machine })
        }

        self.machines
            .push((M::SCHEMA, build_adapter::<M>, TypeId::of::<M>()));

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
    pub entries: HashMap<String, ModbusRtuEntry>,
}

pub struct ModbusRtuEntry {
    pub ident: MachineIdentificationUnique,
    pub init: Box<dyn Fn(String) -> Result<Rc<RefCell<dyn ModbusDevice + 'static>>, String> + Send>,
}
