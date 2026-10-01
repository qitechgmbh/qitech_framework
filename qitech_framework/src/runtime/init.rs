use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;

use qitech_framework_core::report::EtherCATStatus;
use qitech_framework_core::report::ResourceKind;
use qitech_framework_core::report::RuntimeInitEvent;
use qitech_framework_core::report::XtremModuleMetadata;
use qitech_framework_core::schema::MachineSchema;
use qitech_framework_core::session::RuntimeSessionProvider;
use qitech_framework_core::session::RuntimeTransport;
use qitech_framework_core::session::runtime::SessionInitializing;
use qitech_lib::ethercat_hal;
use qitech_lib::xtrem::XtremBusHandle;

use crate::machine::Hardware;
use crate::machine::hardware::ModbusRTUDeviceIdentified;
use crate::machine::hardware::XtremDeviceIdentified;
use crate::resource::Journals;
use crate::resource::PropertyRegistry;
use crate::resource::ResourceRegistry;
use crate::runtime::utils;
use crate::runtime::MachineRegistry;
use crate::runtime::Runtime;
use crate::runtime::RuntimeConfiguration;
use crate::runtime::config::EtherCATMode;
use crate::runtime::config::MachineRegistration;
use crate::runtime::config::ModbusRtuMode;
use crate::runtime::config::XtremMode;
use crate::runtime::error::RuntimeInitializeError;
use crate::runtime::error::RuntimeInitializeResult;
use crate::runtime::ethercat;
use crate::runtime::modbus_rtu;
use crate::runtime::types::HardwareRegistry;
use crate::runtime::types::MachineInstance;
use crate::runtime::types::MachineRegistryEntry;
use crate::runtime::xtrem;

impl<T: RuntimeTransport> Runtime<T> {
    pub fn init<P: RuntimeSessionProvider<Transport = T>>(
        config: RuntimeConfiguration,
        mut provider: P,
    ) -> RuntimeInitializeResult<Self> {
        let session = provider
            .provide()
            .map_err(RuntimeInitializeError::CreateSession)?;

        let mut config_mode = false;

        // --- send hello ---
        let mut session = session.begin_sync()?;

        // --- create machine registry ---
        let mut machine_registry = MachineRegistry::default();

        for MachineRegistration {
            schema,
            build,
            type_id,
            type_name,
        } in config.machines
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

            session.sync(schema)?;
        }

        // --- initialize ethercat ---
        let mut session = session.begin_initialization()?;
        let mut hardware_registry = HashMap::new();

        let (ecat_controller, mut sub_devices) =
            if let EtherCATMode::Enabled(config) = &config.ethercat_mode {
                config_mode = config.stay_preop;
                ethercat::init(config, &mut session, &mut hardware_registry)?
            } else {
                (None, Vec::default())
            };

        // --- initialize modbus rtu ---
        let mut modbus_bindings = HashMap::new();

        let modbus_config = match config.modbus_rtu_mode {
            ModbusRtuMode::Enabled(config) => Some(config),
            _ => None,
        };

        if let Some(modbus_config) = &modbus_config {
            session.send_event(RuntimeInitEvent::ModbusRTUDiscoveryStarted)?;

            for (binding, entry) in &modbus_config.entries {
                let Some(path) = modbus_rtu::resolve_serial_by_path(binding) else {
                    modbus_bindings.insert(binding.clone(), (entry.ident, None));
                    session.send_event(RuntimeInitEvent::ModbusRTUDeviceNotFound {
                        path: binding.clone(),
                    })?;
                    continue;
                };

                modbus_bindings.insert(binding.clone(), (entry.ident, Some(path.clone())));

                let dev_path = path.clone();
                let result = (entry.init)(dev_path);

                let device = match result {
                    Ok(v) => v,
                    Err(e) => {
                        session.send_event(RuntimeInitEvent::ModbusRTUCouldNotInitialize {
                            error: e.to_string(),
                        })?;

                        continue;
                    }
                };

                hardware_registry
                    .entry(entry.ident)
                    .or_insert_with(Vec::new)
                    .push(Hardware::ModbusRTU(ModbusRTUDeviceIdentified {
                        device,
                        binding: binding.clone(),
                    }));
            }
        }

        let modbus = if modbus_bindings.is_empty() {
            None
        } else {
            match modbus_rtu::ModbusManager::new(modbus_bindings) {
                Ok(v) => Some(v),
                Err(e) => {
                    tracing::error!(%e, "failed to spawn modbus rtu watcher");
                    None
                }
            }
        };

        // --- initialize xtrem ---
        let xtrem_bus = Self::init_xtrem(config.xtrem_mode, &mut session, &mut hardware_registry)?;

        // --- build machines ---
        session.send_event(RuntimeInitEvent::BuildingMachines)?;

        let export_count = Rc::new(Cell::new(0));

        let mut journals = Journals::default();

        let mut resources = ResourceRegistry {
            config_properties: PropertyRegistry::new(ResourceKind::ConfigProperty, 4096),
            state_properties: PropertyRegistry::new(ResourceKind::StateProperty, 4096),
            measurements: PropertyRegistry::new(ResourceKind::Measurement, 4096),
        };

        let mut machine_instances: Vec<MachineInstance> = Vec::new();

        let build_outcomes = if config_mode {
            Default::default()
        } else {
            utils::build_machines(
                export_count.clone(),
                &machine_registry,
                &hardware_registry,
                ecat_controller.as_ref().map(|v| v.channel.clone()),
                &mut journals,
                &mut resources,
                &mut machine_instances,
            )
        };

        // --- finalize ethercat ---
        if let Some(controller) = &ecat_controller {
            let EtherCATMode::Enabled(cfg) = &config.ethercat_mode else {
                unreachable!("Cannot create controller without config");
            };

            let state = match controller.app_handle.get_state() {
                ethercat_hal::EtherCATState::NoInterface => EtherCATStatus::NoInterface,
                ethercat_hal::EtherCATState::Boot => EtherCATStatus::Boot,
                ethercat_hal::EtherCATState::Init => EtherCATStatus::Init,
                ethercat_hal::EtherCATState::PreOp => EtherCATStatus::PreOp,
                ethercat_hal::EtherCATState::PreopPdi => EtherCATStatus::PreopPdi,
                ethercat_hal::EtherCATState::Op => EtherCATStatus::Op,
            };
            session.send_event(RuntimeInitEvent::EtherCATStateUpdate(state))?;

            if !cfg.stay_preop {
                session.send_event(RuntimeInitEvent::EtherCATFinalizing)?;
                ethercat::finalize(controller, &mut sub_devices)?;

                let state = match controller.app_handle.get_state() {
                    ethercat_hal::EtherCATState::NoInterface => EtherCATStatus::NoInterface,
                    ethercat_hal::EtherCATState::Boot => EtherCATStatus::Boot,
                    ethercat_hal::EtherCATState::Init => EtherCATStatus::Init,
                    ethercat_hal::EtherCATState::PreOp => EtherCATStatus::PreOp,
                    ethercat_hal::EtherCATState::PreopPdi => EtherCATStatus::PreopPdi,
                    ethercat_hal::EtherCATState::Op => EtherCATStatus::Op,
                };
                session.send_event(RuntimeInitEvent::EtherCATStateUpdate(state))?;
            }
        }

        // --- announce machine build results, now that the bus is confirmed Op
        //     (or immediately, if ethercat is disabled entirely) ---
        for (ident, result) in build_outcomes {
            session.send_event(RuntimeInitEvent::MachineBuildCompleted { ident, result })?;
        }

        // --- return initialized runtime ---
        let mut rt = Runtime {
            machine_registry,
            hardware_registry,
            export_count,
            journals,
            resources,
            report: Default::default(),
            machine_instances,
            sub_devices,
            ecat_controller,
            _xtrem_bus: xtrem_bus,
            modbus,
            modbus_config,
            config: config.config,
            session: session.upgrade()?,

            // set into future so first export always succeeds
            last_export_ts: Instant::now() - Duration::from_secs(420),
            config_mode,
        };

        // --- send report with all registered resources and machines ---
        rt.export_report_if_due(Instant::now());

        // --- and yield the runtime finally ---
        Ok(rt)
    }

    /// Open the shared XTREM bus
    fn init_xtrem(
        mode: XtremMode,
        session: &mut SessionInitializing<T>,
        hardware_registry: &mut HardwareRegistry,
    ) -> RuntimeInitializeResult<Option<XtremBusHandle>> {
        let XtremMode::Enabled(config) = mode else {
            return Ok(None);
        };

        session.send_event(RuntimeInitEvent::XtremDiscoveryStarted)?;

        let handle = match xtrem::open_bus(&config) {
            Ok(v) => v,
            Err(e) => {
                session.send_event(RuntimeInitEvent::XtremBusFailed {
                    error: e.to_string(),
                })?;

                return Ok(None);
            }
        };

        let probes = match xtrem::discover(&handle, config.discovery_window) {
            Ok(v) => v,
            Err(e) => {
                session.send_event(RuntimeInitEvent::XtremBusFailed {
                    error: e.to_string(),
                })?;

                return Ok(None);
            }
        };

        // --- report everything that answered, claimed or not ---
        session.send_event(RuntimeInitEvent::XtremDiscoveryCompleted {
            modules: probes
                .iter()
                .map(|probe| XtremModuleMetadata {
                    serial: probe.serial,
                    device_id: probe.device_id,
                    addr: probe.addr.to_string(),
                    id_collision: probe.id_collision,
                })
                .collect(),
        })?;

        for (device_id, entry) in config.entries {
            let Some(probe) = probes.iter().find(|probe| probe.device_id == device_id) else {
                session.send_event(RuntimeInitEvent::XtremDeviceNotFound { device_id })?;
                continue;
            };

            if probe.id_collision {
                session.send_event(RuntimeInitEvent::XtremDeviceIdCollision { device_id })?;
                continue;
            }

            let device = match (entry.init)(&handle, probe) {
                Ok(v) => v,
                Err(error) => {
                    session.send_event(RuntimeInitEvent::XtremCouldNotInitialize {
                        device_id,
                        error,
                    })?;

                    continue;
                }
            };

            hardware_registry
                .entry(entry.ident)
                .or_default()
                .push(Hardware::Xtrem(XtremDeviceIdentified {
                    device,
                    probe: probe.clone(),
                }));
        }

        Ok(Some(handle))
    }
}
