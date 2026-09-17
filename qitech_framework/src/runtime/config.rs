use std::any::TypeId;
use std::any::type_name;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use qitech_framework_core::ident::MachineInstanceIdentification;
use qitech_framework_core::report::error::BuildError;
use qitech_lib::ethercat_hal::MasterConfiguration;
use qitech_lib::xtrem::ScaleMode;
use qitech_lib::xtrem::XtremBusConfig;
use qitech_lib::xtrem::XtremBusHandle;
use qitech_lib::xtrem::XtremDevice;
use qitech_lib::xtrem::XtremProbe;
use qitech_lib::xtrem::discovery;

use crate::machine::BuildContext;
use crate::machine::Machine;
use crate::machine::MachineBuild;
use crate::machine::MachineDescriptor;
use crate::modbus::ModbusRTUBusConfig;
use crate::runtime::types::BuildMachineFn;
use crate::runtime::types::Config;
use crate::runtime::xtrem::XtremDeviceBuild;

#[derive(Default)]
pub struct RuntimeConfiguration {
    pub(crate) config: Config,
    pub(crate) machines: Vec<MachineRegistration>,
    pub(crate) ethercat_mode: EtherCATMode,
    pub(crate) xtrem_mode: XtremMode,

    pub(crate) modbus_rtu_buses: Vec<ModbusRTUBusConfig>,
}

impl RuntimeConfiguration {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn when<F>(mut self, condition: bool, f: F) -> Self
    where
        F: FnOnce(&mut Self),
    {
        if condition {
            f(&mut self);
        }

        self
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

    pub fn modbus_rtu_bus(mut self, config: ModbusRTUBusConfig) -> Self {
        // TODO: move to build process
        // assert!(!self.modbus_rtu_buses.iter().any(|existing| existing.port == config.port));
        self.modbus_rtu_buses.push(config);
        self
    }

    pub fn xtrem(mut self, config: XtremConfig) -> Self {
        let entries = match self.xtrem_mode {
            XtremMode::Enabled(previous) => previous.entries,
            _ => Default::default(),
        };

        self.xtrem_mode = XtremMode::Enabled(XtremConfig { entries, ..config });
        self
    }

    pub fn xtrem_device<D: XtremDeviceBuild + 'static>(
        mut self,
        device_id: u8,
        ident: MachineInstanceIdentification,
        mode: ScaleMode,
    ) -> Self {
        let mut config = match self.xtrem_mode {
            XtremMode::Enabled(config) => config,
            _ => XtremConfig::default(),
        };

        let init = Box::new(move |bus: &XtremBusHandle, probe: &XtremProbe| {
            let dev = D::build(bus, probe, mode).map_err(|e| format!("{e}"))?;
            let dev: Rc<RefCell<dyn XtremDevice>> = Rc::new(RefCell::new(dev));
            Ok(dev)
        });

        config.entries.insert(device_id, XtremEntry { ident, init });

        self.xtrem_mode = XtremMode::Enabled(config);
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
pub enum XtremMode {
    #[default]
    Disabled,
    Enabled(XtremConfig),

    #[allow(unused)]
    Mock,
}

pub struct XtremConfig {
    pub bus: XtremBusConfig,
    pub discovery_window: Duration,

    /// Keyed by module device id (register `0001h`).
    pub entries: HashMap<u8, XtremEntry>,
}

impl Default for XtremConfig {
    fn default() -> Self {
        Self {
            bus: Default::default(),
            discovery_window: discovery::DEFAULT_DISCOVERY_WINDOW,
            entries: Default::default(),
        }
    }
}

pub struct XtremEntry {
    pub ident: MachineInstanceIdentification,
    pub init: NewXtremDeviceFn,
}

pub(crate) struct MachineRegistration {
    pub schema: &'static str,
    pub build: BuildMachineFn,
    pub type_id: TypeId,
    pub type_name: &'static str,
}

pub type NewXtremDeviceFn = Box<
    dyn Fn(&XtremBusHandle, &XtremProbe) -> Result<Rc<RefCell<dyn XtremDevice + 'static>>, String>
        + Send,
>;
