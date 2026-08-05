use std::collections::HashMap;
use std::time::Instant;

use qitech_framework_core::report::EtherCATStatus;
use qitech_framework_core::report::RuntimeInitEvent;
use qitech_framework_core::schema::MachineSchema;
use qitech_framework_core::session::RuntimeTransport;
use qitech_framework_core::session::runtime::SessionHandshake;
use qitech_framework_core::session::runtime::SessionInitializing;
use qitech_lib::ethercat_hal;
use qitech_lib::ethercat_hal::EtherCATThreadChannel;

use crate::Runtime;
use crate::machine::BuildContext;
use crate::machine::Hardware;
use crate::machine::hardware::ModbusRTUDeviceIdentified;
use crate::resource::ConfigPropertyRegistry;
use crate::resource::Journals;
use crate::runtime::MachineRegistry;
use crate::runtime::RuntimeConfiguration;
use crate::runtime::RuntimeStatus;
use crate::runtime::config::EtherCATMode;
use crate::runtime::config::ModbusRtuMode;
use crate::runtime::error::RuntimeInitializeError;
use crate::runtime::error::RuntimeInitializeResult;
use crate::runtime::ethercat;
use crate::runtime::modbus_rtu;
use crate::runtime::types::HardwareRegistry;
use crate::runtime::types::MachineInstance;
use crate::runtime::types::MachineRegistryEntry;

impl<T: RuntimeTransport> Runtime<T> {
    pub fn init(
        config: RuntimeConfiguration,
        session: SessionHandshake<T>,
    ) -> RuntimeInitializeResult<Self> {
        // --- send hello ---
        let mut session = session.complete()?;

        // --- create machine registry ---
        let mut machine_registry = MachineRegistry::default();

        for (schema_str, build_fn, type_id) in config.machines {
            let schema = MachineSchema::parse_str(schema_str)?;

            if machine_registry
                .insert(
                    schema.identification,
                    MachineRegistryEntry {
                        build: build_fn,
                        type_id,
                    },
                )
                .is_some()
            {
                return Err(RuntimeInitializeError::DuplicateMachine(
                    schema.identification,
                ));
            }

            session.sync(schema.clone())?;
        }

        // --- initialize ethercat ---
        let mut session = session.complete()?;
        let mut hardware_registry = HashMap::new();

        let (ecat_controller, mut sub_devices) =
            if let EtherCATMode::Enabled(config) = config.ethercat_mode {
                ethercat::init(config, &mut session, &mut hardware_registry)?
            } else {
                (None, Vec::default())
            };

        // --- initialize modbus rtu ---
        if let ModbusRtuMode::Enabled(config) = config.modbus_rtu_mode {
            session.send_event(RuntimeInitEvent::ModbusRTUDiscoveryStarted)?;

            for (path, entry) in config.entries {
                let Some(path) = modbus_rtu::resolve_serial_by_path(&path) else {
                    session.send_event(RuntimeInitEvent::ModbusRTUDeviceNotFound { path })?;
                    continue;
                };

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
                        path,
                    }));
            }
        }

        // --- build machines ---
        session.send_event(RuntimeInitEvent::BuildingMachines)?;

        // let mut resources = Box::new(Resources::default());
        let mut journals = Journals::default();
        let mut config_properties = ConfigPropertyRegistry::new(4096, 128);

        let machines = Self::init_machines(
            &mut session,
            &machine_registry,
            &hardware_registry,
            ecat_controller.as_ref().map(|v| v.channel.clone()),
            &mut journals,
            &mut config_properties,
        )?;

        // --- finalize ethercat ---
        if let Some(controller) = &ecat_controller {
            let state = match controller.app_handle.get_state() {
                ethercat_hal::EtherCATState::NoInterface => EtherCATStatus::NoInterface,
                ethercat_hal::EtherCATState::Boot => EtherCATStatus::Boot,
                ethercat_hal::EtherCATState::Init => EtherCATStatus::Init,
                ethercat_hal::EtherCATState::PreOp => EtherCATStatus::PreOp,
                ethercat_hal::EtherCATState::PreopPdi => EtherCATStatus::PreopPdi,
                ethercat_hal::EtherCATState::Op => EtherCATStatus::Op,
            };
            session.send_event(RuntimeInitEvent::EtherCATStateUpdate(state))?;

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

        // --- return initialized runtime ---
        session.send_event(RuntimeInitEvent::Finished)?;

        Ok(Runtime {
            status: RuntimeStatus::Initialized,
            // machine_registry,
            // hardware_registry,
            export_cycle: 0,
            journals,
            report: Default::default(),
            machines,
            sub_devices,
            ecat_controller,
            config: config.config,
            session: session.complete()?,
            last_export_ts: Instant::now(),
            subscriptions: Default::default(),
        })
    }

    fn init_machines(
        session: &mut SessionInitializing<T>,
        machine_registry: &MachineRegistry,
        hardware_registry: &HardwareRegistry,
        ecat_interface: Option<EtherCATThreadChannel>,
        journals: &mut Journals,
        config_properties: &mut ConfigPropertyRegistry,
    ) -> RuntimeInitializeResult<Vec<MachineInstance>> {
        let mut machines: Vec<MachineInstance> = Vec::new();

        for (ident_unique, hardware) in hardware_registry {
            let ident = ident_unique.identification;

            let Some(entry) = machine_registry.get(&ident) else {
                session.send_event(RuntimeInitEvent::FailedToBuildMachine {
                    ident: *ident_unique,
                })?;

                continue;
            };

            let ctx = BuildContext::new(
                *ident_unique,
                entry.type_id,
                ecat_interface.clone(),
                hardware.clone(),
                journals,
                config_properties.begin_commit(*ident_unique),
            );

            let instance = match (entry.build)(ctx) {
                Ok(v) => v,
                Err(_) => {
                    session.send_event(RuntimeInitEvent::FailedToBuildMachine {
                        ident: *ident_unique,
                    })?;

                    continue;
                }
            };

            machines.push(instance);
            session.send_event(RuntimeInitEvent::BuiltMachine {
                ident: *ident_unique,
            })?;
        }

        Ok(machines)
    }
}
