use std::any::TypeId;
use std::any::type_name;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_framework_core::report::error::BuildError;
use qitech_lib::ethercat_hal::MasterConfiguration;
use qitech_lib::modbus::ModbusDevice;
use qitech_lib::modbus::ModbusSettings;
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
use crate::runtime::types::BuildMachineFn;
use crate::runtime::types::Config;
use crate::runtime::xtrem::XtremDeviceBuild;

#[derive(Default)]
pub struct RuntimeConfiguration {
    pub(crate) config: Config,
    pub(crate) machines: Vec<MachineRegistration>,
    pub(crate) ethercat_mode: EtherCATMode,
    pub(crate) modbus_rtu_mode: ModbusRtuMode,
    pub(crate) xtrem_mode: XtremMode,
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

    /// Set the bus-level XTREM settings — bind address, broadcast address, discovery window.
    ///
    /// Devices are registered separately with [`RuntimeConfiguration::xtrem_device`]. Any that
    /// were already registered are kept, so the two can be called in either order.
    pub fn xtrem(mut self, config: XtremConfig) -> Self {
        let entries = match self.xtrem_mode {
            XtremMode::Enabled(previous) => previous.entries,
            _ => Default::default(),
        };

        self.xtrem_mode = XtremMode::Enabled(XtremConfig { entries, ..config });
        self
    }

    /// Claim the module with the given `device_id` for `ident`.
    ///
    /// `device_id` is register `0001h` — the address the bus routes replies on, set per module
    /// with the `assign_ids` tool. Init resolves it to a live module through one broadcast
    /// discovery sweep, which also supplies the unicast address.
    ///
    /// Modules ship as `01`, so give each one a distinct id before configuring it here. Two
    /// modules sharing an id cannot be told apart and init refuses both rather than guessing.
    ///
    /// One call claims one module for one machine instance, so N scales become N machines.
    pub fn xtrem_device<D: XtremDeviceBuild + 'static>(
        mut self,
        device_id: u8,
        ident: MachineIdentificationUnique,
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
    pub init: NewModbusDeviceFn,
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
    pub ident: MachineIdentificationUnique,
    pub init: NewXtremDeviceFn,
}

pub(crate) struct MachineRegistration {
    pub schema: &'static str,
    pub build: BuildMachineFn,
    pub type_id: TypeId,
    pub type_name: &'static str,
}

pub type NewModbusDeviceFn =
    Box<dyn Fn(String) -> Result<Rc<RefCell<dyn ModbusDevice + 'static>>, String> + Send>;

pub type NewXtremDeviceFn = Box<
    dyn Fn(&XtremBusHandle, &XtremProbe) -> Result<Rc<RefCell<dyn XtremDevice + 'static>>, String>
        + Send,
>;
