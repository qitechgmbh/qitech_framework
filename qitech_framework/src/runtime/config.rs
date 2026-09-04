use std::any::TypeId;
use std::any::type_name;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use qitech_framework_core::ident::MachineIdentification;
use qitech_framework_core::report::error::BuildError;
use qitech_lib::ethercat_hal::MasterConfiguration;
use qitech_lib::modbus::ModbusDevice;
use qitech_lib::modbus::ModbusSettings;

use crate::machine::BuildContext;
use crate::machine::Machine;
use crate::machine::MachineBuild;
use crate::machine::MachineDescriptor;
use crate::runtime::types::BuildMachineFn;
use crate::runtime::types::Config;

#[derive(Default)]
pub struct RuntimeConfiguration {
    pub(crate) config: Config,
    pub(crate) machines: Vec<MachineRegistration>,
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

    pub fn cycle_period(mut self, value: Duration) -> Self {
        self.config.cycle_period = value;
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

    /// Register a driver for a machine *type*: "machines of type `machine` are driven over
    /// Modbus RTU by `D`". The USB serial port and machine instance (serial number) are not
    /// known at compile time - they come from the user-configured assignments persisted via
    /// `runtime::modbus_rtu`, matched up at runtime init.
    pub fn modbus_rtu_driver<D: ModbusDevice + 'static>(
        mut self,
        machine: MachineIdentification,
        default_slave_id: u8,
        settings: Option<ModbusSettings>,
    ) -> Self {
        let mut config = match self.modbus_rtu_mode {
            ModbusRtuMode::Enabled(config) => config,
            _ => ModbusRtuConfig {
                drivers: Default::default(),
            },
        };

        let init = Box::new(move |path: String, slave_id: u8| {
            let dev = D::new(path, slave_id, settings).map_err(|e| format!("{e}"))?;
            let dev: Rc<RefCell<dyn ModbusDevice>> = Rc::new(RefCell::new(dev));
            Ok(dev)
        });

        config.drivers.insert(
            machine,
            ModbusRtuDriver {
                init,
                default_slave_id,
            },
        );

        self.modbus_rtu_mode = ModbusRtuMode::Enabled(config);
        self
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
    pub master_config: MasterConfiguration,
}

impl Default for EtherCATConfig {
    fn default() -> Self {
        Self {
            interface_scan_interval: Duration::from_secs(2),
            master_config: Default::default(),
        }
    }
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
    pub drivers: HashMap<MachineIdentification, ModbusRtuDriver>,
}

pub struct ModbusRtuDriver {
    pub init: NewModbusDeviceFn,
    pub default_slave_id: u8,
}

pub(crate) struct MachineRegistration {
    pub schema: &'static str,
    pub build: BuildMachineFn,
    pub type_id: TypeId,
    pub type_name: &'static str,
}

pub type NewModbusDeviceFn =
    Box<dyn Fn(String, u8) -> Result<Rc<RefCell<dyn ModbusDevice + 'static>>, String> + Send>;
