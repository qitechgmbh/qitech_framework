use std::cell::RefCell;
use std::println;
use std::rc::Rc;
use std::time::Instant;

use qitech_framework_common::MachineSchema;
use qitech_framework_common::RuntimeInitEvent;
use qitech_lib::ethercat_hal::EtherCATThreadChannel;
use qitech_lib::modbus::ModbusDevice;
use qitech_lib::modbus::devices::qitech_laser::LaserDevice;

use crate::Runtime;
use crate::machine::BuildContext;
use crate::machine::Hardware;
use crate::machine::Resources;
use crate::machine::hardware::ModbusRTUDeviceIdentified;
use crate::runtime::MachineRegistry;
use crate::runtime::RuntimeConfiguration;
use crate::runtime::RuntimeSession;
use crate::runtime::RuntimeStatus;
use crate::runtime::bridge::RuntimeSessionHandshake;
use crate::runtime::config::EtherCATMode;
use crate::runtime::config::ModbusRtuMode;
use crate::runtime::error::RuntimeInitializeError;
use crate::runtime::error::RuntimeInitializeResult;
use crate::runtime::ethercat;
use crate::runtime::modbus_rtu;
use crate::runtime::types::HardwareRegistry;
use crate::runtime::types::MachineInstance;

impl<S: RuntimeSession> Runtime<S> {
    pub fn init(
        config: RuntimeConfiguration,
        mut handshake: S::Handshake,
    ) -> RuntimeInitializeResult<Self> {
        // --- send hello ---
        handshake.send_hello()?;

        // --- create machine registry ---
        let mut machine_registry = MachineRegistry::default();

        for (schema_str, build_fn) in config.machines {
            let schema = MachineSchema::from_yaml_str(schema_str)?;

            if machine_registry
                .insert(schema.identification, build_fn)
                .is_some()
            {
                return Err(RuntimeInitializeError::DuplicateMachine(
                    schema.identification,
                ));
            }

            handshake.sync_machine(schema_str)?;
        }

        // --- initialize ethercat ---
        let mut hardware_registry = Default::default();

        let (ecat_controller, mut sub_devices) =
            if let EtherCATMode::Enabled(config) = config.ethercat_mode {
                ethercat::init::<S>(config, &mut handshake, &mut hardware_registry)?
            } else {
                (None, Default::default())
            };

        // --- initialize modbus rtu ---
        if let ModbusRtuMode::Enabled(config) = config.modbus_rtu_mode {
            handshake.submit_event(RuntimeInitEvent::ModbusDiscoveryStarted)?;

            for (path, ident) in config.bindings {
                let Some(path) = modbus_rtu::resolve_serial_by_path(&path) else {
                    // path not found
                    continue;
                };

                let device: Rc<RefCell<LaserDevice>> = Rc::new(RefCell::new(
                    LaserDevice::new(path.clone(), 1, None).unwrap(),
                ));

                hardware_registry.insert(
                    ident,
                    vec![Hardware::ModbusRTU(ModbusRTUDeviceIdentified {
                        device,
                        path,
                    })],
                );
            }
        }

        // --- build machines ---
        handshake.submit_event(RuntimeInitEvent::BuildingMachines)?;

        let mut resources = Box::new(Resources::default());
        let machines = Self::init_machines(
            &mut handshake,
            &machine_registry,
            &hardware_registry,
            ecat_controller.as_ref().map(|v| v.channel.clone()),
            &mut resources,
        )?;

        // --- finalize ethercat ---
        if let Some(controller) = &ecat_controller {
            handshake.submit_event(RuntimeInitEvent::EtherCATFinalizing)?;
            ethercat::finalize(controller, &mut sub_devices)?;
        }

        // --- return initialized runtime ---
        Ok(Runtime {
            status: RuntimeStatus::Initialized,
            // machine_registry,
            // hardware_registry,
            resources,
            report: Default::default(),
            machines,
            sub_devices,
            ecat_controller,
            config: config.config,
            bridge: handshake.complete()?,
            last_export_ts: Instant::now(),
            subscriptions: Default::default(),
        })
    }

    fn init_machines(
        bridge: &mut S::Handshake,
        machine_registry: &MachineRegistry,
        hardware_registry: &HardwareRegistry,
        ecat_interface: Option<EtherCATThreadChannel>,
        resources: &mut Resources,
    ) -> RuntimeInitializeResult<Vec<MachineInstance>> {
        let mut machines: Vec<MachineInstance> = Vec::new();

        for (ident_unique, hardware) in hardware_registry {
            let ident = ident_unique.identification;

            let Some(build) = machine_registry.get(&ident) else {
                println!("Failed to build machine `{ident_unique}`. No entry");
                continue;
                // bail!("Failed to find registry entry for machine {{{ident}}}");
            };

            let ctx = BuildContext::new(
                *ident_unique,
                ecat_interface.clone(),
                resources,
                hardware.clone(),
            );

            println!("Building machine `{ident_unique}`");

            let inner = match (build)(ctx) {
                Ok(v) => v,
                Err(e) => {
                    _ = e;
                    println!("Failed to build machine: {e}");
                    continue;
                }
            };

            machines.push(MachineInstance {
                ident: *ident_unique,
                inner,
            });

            bridge.submit_event(RuntimeInitEvent::BuiltMachine {
                ident: *ident_unique,
            })?;
        }

        Ok(machines)
    }
}
