use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;

use qitech_framework_core::report::EtherCATStatus;
use qitech_framework_core::report::ResourceKind;
use qitech_framework_core::report::RuntimeInitEvent;
use qitech_framework_core::report::error::BuildError;
use qitech_framework_core::schema::MachineSchema;
use qitech_framework_core::session::RuntimeTransport;
use qitech_framework_core::session::runtime::SessionHandshake;
use qitech_framework_core::session::runtime::SessionInitializing;
use qitech_lib::ethercat_hal;
use qitech_lib::ethercat_hal::EtherCATThreadChannel;

use crate::Runtime;
use crate::journal::Journals;
use crate::machine::BuildContext;
use crate::machine::Hardware;
use crate::machine::MachineInstance;
use crate::machine::PropertyRegistry;
use crate::machine::ResourceRegistry;
use crate::machine::hardware::ModbusRTUDeviceIdentified;
use crate::runtime::MachineRegistry;
use crate::runtime::RuntimeConfiguration;
use crate::runtime::config::EtherCATMode;
use crate::runtime::config::MachineRegistration;
use crate::runtime::config::ModbusRtuMode;
use crate::runtime::error::RuntimeInitializeError;
use crate::runtime::error::RuntimeInitializeResult;
use crate::runtime::ethercat;
use crate::runtime::modbus_rtu;
use crate::runtime::types::HardwareRegistry;
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

        for MachineRegistration {
            schema,
            build,
            type_id,
            type_name,
        } in config.machines
        {
            let schema = MachineSchema::parse_str(schema)?;

            if machine_registry
                .insert(
                    schema.identification,
                    MachineRegistryEntry {
                        build,
                        type_id,
                        type_name,
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

        let mut journals = Journals::default();

        let mut resources = ResourceRegistry {
            config_properties: PropertyRegistry::new(ResourceKind::ConfigProperty, 4096),
            state_properties: PropertyRegistry::new(ResourceKind::StateProperty, 4096),
            measurements: PropertyRegistry::new(ResourceKind::Measurement, 4096),
        };

        let machines = Self::init_machines(
            &mut session,
            &machine_registry,
            &hardware_registry,
            ecat_controller.as_ref().map(|v| v.channel.clone()),
            &mut journals,
            &mut resources,
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

        let mut rt = Runtime {
            // machine_registry,
            // hardware_registry,
            journals,
            resources,
            report: Default::default(),
            machines,
            sub_devices,
            ecat_controller,
            config: config.config,
            session: session.complete()?,

            // set into future so first export always succeeds - nice
            last_export_ts: Instant::now() - Duration::from_secs(420),
        };

        // --- send report with all registered resources and machines ---
        rt.export_report_if_due(Instant::now());

        // --- and yield the runtime finally ---
        Ok(rt)
    }

    fn init_machines(
        session: &mut SessionInitializing<T>,
        machine_registry: &MachineRegistry,
        hardware_registry: &HardwareRegistry,
        ecat_interface: Option<EtherCATThreadChannel>,
        journals: &mut Journals,
        resources: &mut ResourceRegistry,
    ) -> RuntimeInitializeResult<Vec<MachineInstance>> {
        let mut machines: Vec<MachineInstance> = Vec::new();

        for (ident_unique, hardware) in hardware_registry {
            let ident = ident_unique.identification;

            let Some(entry) = machine_registry.get(&ident) else {
                session.send_event(RuntimeInitEvent::MachineBuildCompleted {
                    ident: *ident_unique,
                    result: Err(BuildError::MachineTypeNotRegistered),
                })?;

                continue;
            };

            let mut ctx = BuildContext {
                ident: *ident_unique,
                type_id: entry.type_id,
                type_name: entry.type_name,
                ethercat_interface: ecat_interface.clone(),
                hardware: hardware.clone(),
                journals,
                config: resources.config_properties.register(),
                state: resources.state_properties.register(),
                measurements: resources.measurements.register(),
                journals_temp: Journals::default(),
                config_registered: Default::default(),
                state_registered: Default::default(),
                measurements_registered: Default::default(),
                commands_registered: Default::default(),
            };

            let machine = match (entry.build)(&mut ctx) {
                Ok(v) => v,
                Err(e) => {
                    session.send_event(RuntimeInitEvent::MachineBuildCompleted {
                        ident: *ident_unique,
                        result: Err(e),
                    })?;

                    continue;
                }
            };

            // --- commit allocations ---
            ctx.config.commit();
            ctx.state.commit();
            ctx.measurements.commit();

            // --- merge records from temp journal into export journal ---
            let journal = ctx.journals.config_property.new_handle();
            ctx.journals_temp.config_property.drain_with(|record| {
                journal.append(record);
            });

            let journal = ctx.journals.state_property.new_handle();
            ctx.journals_temp.state_property.drain_with(|record| {
                journal.append(record);
            });

            let journal = ctx.journals.commands.new_handle();
            ctx.journals_temp.commands.drain_with(|record| {
                journal.append(record);
            });

            // --- extract metadata ---
            let configs = ctx.config_registered;
            let commands = ctx.commands_registered;

            machines.push(MachineInstance {
                ident: *ident_unique,
                machine,
                configs,
                commands,
                subscriptions: Default::default(),
            });

            session.send_event(RuntimeInitEvent::MachineBuildCompleted {
                ident: *ident_unique,
                result: Ok(()),
            })?;
        }

        Ok(machines)
    }
}
