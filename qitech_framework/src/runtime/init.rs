use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;

use qitech_framework_core::ident::MachineInstanceIdentification;
use qitech_framework_core::report::EtherCATStatus;
use qitech_framework_core::report::ResourceKind;
use qitech_framework_core::report::RuntimeInitEvent;
use qitech_framework_core::report::error::BuildError;
use qitech_framework_core::schema::MachineSchema;
use qitech_framework_core::session::RuntimeSessionProvider;
use qitech_framework_core::session::RuntimeTransport;
use qitech_lib::ethercat_hal;
use qitech_lib::ethercat_hal::EtherCATThreadChannel;

use crate::machine::BuildContext;
use crate::machine::Hardware;
use crate::machine::hardware::ModbusRTUDeviceIdentified;
use crate::resource::Journals;
use crate::resource::PropertyRegistry;
use crate::resource::ResourceRegistry;
use crate::runtime::MachineRegistry;
use crate::runtime::Runtime;
use crate::runtime::RuntimeConfiguration;
use crate::runtime::config::EtherCATMode;
use crate::runtime::config::MachineRegistration;
use crate::runtime::config::ModbusRtuMode;
use crate::runtime::error::RuntimeInitializeError;
use crate::runtime::error::RuntimeInitializeResult;
use crate::runtime::ethercat;
use crate::runtime::modbus_rtu;
use crate::runtime::types::HardwareRegistry;
use crate::runtime::types::MachineInstance;
use crate::runtime::types::MachineRegistryEntry;

impl<T: RuntimeTransport> Runtime<T> {
    pub fn init<P: RuntimeSessionProvider<Transport = T>>(
        config: RuntimeConfiguration,
        mut provider: P,
    ) -> RuntimeInitializeResult<Self> {
        let session = provider
            .provide()
            .map_err(RuntimeInitializeError::CreateSession)?;

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
            if let EtherCATMode::Enabled(config) = config.ethercat_mode {
                ethercat::init(config, &mut session, &mut hardware_registry)?
            } else {
                (None, Vec::default())
            };

        // --- discover modbus rtu ports (always, so the Setup page has something to show even
        //     when no driver is registered for this build) ---
        session.send_event(RuntimeInitEvent::ModbusRTUDiscoveryStarted)?;
        let modbus_devices = modbus_rtu::list_modbus_devices();

        // --- initialize assigned modbus rtu devices ---
        if let ModbusRtuMode::Enabled(config) = config.modbus_rtu_mode {
            for device in &modbus_devices {
                let Some(assignment) = &device.assignment else {
                    continue;
                };

                let Some(driver) = config.drivers.get(&assignment.machine.machine) else {
                    session.send_event(RuntimeInitEvent::ModbusRTUNoDriverForMachine {
                        port: device.port.clone(),
                        machine: assignment.machine.machine,
                    })?;
                    continue;
                };

                let Some(node) = &device.device_node else {
                    session.send_event(RuntimeInitEvent::ModbusRTUDeviceNotFound {
                        path: device.port.clone(),
                    })?;
                    continue;
                };

                // a slave id of 0 (the Modbus broadcast address, never a valid target to poll)
                // is used as the "unset" sentinel by the assignment UI, falling back to the
                // driver's own default.
                let slave_id = if assignment.slave_id == 0 {
                    driver.default_slave_id
                } else {
                    assignment.slave_id
                };

                let result = (driver.init)(node.clone(), slave_id);

                let device_handle = match result {
                    Ok(v) => v,
                    Err(e) => {
                        session.send_event(RuntimeInitEvent::ModbusRTUCouldNotInitialize {
                            error: e.to_string(),
                        })?;

                        continue;
                    }
                };

                hardware_registry
                    .entry(assignment.machine)
                    .or_insert_with(Vec::new)
                    .push(Hardware::ModbusRTU(ModbusRTUDeviceIdentified {
                        device: device_handle,
                    }));
            }
        }

        session.send_event(RuntimeInitEvent::ModbusRTUDiscoveryCompleted {
            devices: modbus_devices,
        })?;

        // --- build machines ---
        session.send_event(RuntimeInitEvent::BuildingMachines)?;

        let export_count = Rc::new(Cell::new(0));

        let mut journals = Journals::default();

        let mut resources = ResourceRegistry {
            config_properties: PropertyRegistry::new(ResourceKind::ConfigProperty, 4096),
            state_properties: PropertyRegistry::new(ResourceKind::StateProperty, 4096),
            measurements: PropertyRegistry::new(ResourceKind::Measurement, 4096),
        };

        let (machines, build_outcomes) = Self::init_machines(
            export_count.clone(),
            &machine_registry,
            &hardware_registry,
            ecat_controller.as_ref().map(|v| v.channel.clone()),
            &mut journals,
            &mut resources,
        );

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

        // --- announce machine build results, now that the bus is confirmed Op
        //     (or immediately, if ethercat is disabled entirely) ---
        for (ident, result) in build_outcomes {
            session.send_event(RuntimeInitEvent::MachineBuildCompleted { ident, result })?;
        }

        // --- return initialized runtime ---
        let mut rt = Runtime {
            // machine_registry,
            // hardware_registry,
            export_count,
            journals,
            resources,
            report: Default::default(),
            machines,
            sub_devices,
            ecat_controller,
            config: config.config,
            session: session.upgrade()?,

            // set into future so first export always succeeds - nice
            last_export_ts: Instant::now() - Duration::from_secs(420),
        };

        // --- send report with all registered resources and machines ---
        rt.export_report_if_due(Instant::now());

        // --- and yield the runtime finally ---
        Ok(rt)
    }

    fn init_machines(
        export_count: Rc<Cell<u64>>,
        machine_registry: &MachineRegistry,
        hardware_registry: &HardwareRegistry,
        ecat_interface: Option<EtherCATThreadChannel>,
        journals: &mut Journals,
        resources: &mut ResourceRegistry,
    ) -> (
        Vec<MachineInstance>,
        Vec<(MachineInstanceIdentification, Result<(), BuildError>)>,
    ) {
        let mut machines: Vec<MachineInstance> = Vec::new();
        let mut outcomes: Vec<(MachineInstanceIdentification, Result<(), BuildError>)> =
            Vec::new();

        for (ident_unique, hardware) in hardware_registry {
            let ident = ident_unique.machine;

            let Some(entry) = machine_registry.get(&ident) else {
                outcomes.push((*ident_unique, Err(BuildError::MachineTypeNotRegistered)));

                continue;
            };

            let mut ctx = BuildContext {
                ident: *ident_unique,
                schema: &entry.schema,
                export_count: export_count.clone(),
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
                events_registered: Default::default(),
            };

            let machine = match (entry.build)(&mut ctx) {
                Ok(v) => v,
                Err(e) => {
                    outcomes.push((*ident_unique, Err(e)));

                    continue;
                }
            };

            // --- commit allocations ---
            ctx.config.commit();
            ctx.state.commit();
            ctx.measurements.commit();

            // --- import records from temp journal into export journal ---
            ctx.journals
                .config_property
                .import(ctx.journals_temp.config_property);

            ctx.journals
                .state_property
                .import(ctx.journals_temp.state_property);

            ctx.journals.command.import(ctx.journals_temp.command);
            ctx.journals.event.import(ctx.journals_temp.event);

            // --- extract metadata ---
            let configs = ctx.config_registered;
            let commands = ctx.commands_registered;

            // --- create instance ---
            machines.push(MachineInstance {
                ident: *ident_unique,
                machine,
                configs,
                commands,
                subscriptions: Default::default(),
            });

            // --- record outcome ---
            outcomes.push((*ident_unique, Ok(())));
        }

        (machines, outcomes)
    }
}
