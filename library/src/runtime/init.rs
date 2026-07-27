use std::time::Instant;

use qitech_framework_common::MachineSchema;
use qitech_framework_common::RuntimeInitEvent;
use qitech_lib::ethercat_hal::EtherCATThreadChannel;

use crate::Runtime;
use crate::machine::BuildContext;
use crate::machine::Resources;
use crate::runtime::Bridge;
use crate::runtime::MachineRegistry;
use crate::runtime::RuntimeConfiguration;
use crate::runtime::RuntimeStatus;
use crate::runtime::bridge::BridgeBootstrap;
use crate::runtime::config::EtherCATMode;
use crate::runtime::error::RuntimeInitializeError;
use crate::runtime::error::RuntimeInitializeResult;
use crate::runtime::ethercat;
use crate::runtime::types::HardwareRegistry;
use crate::runtime::types::MachineInstance;

impl<B: Bridge> Runtime<B> {
    pub fn init(
        config: RuntimeConfiguration,
        mut bootstrap: B::Bootstrap,
    ) -> RuntimeInitializeResult<Self> {
        // --- send hello ---
        bootstrap.send_hello()?;

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

            bootstrap.sync_machine(schema_str)?;
        }

        // --- initialize ethercat ---
        let mut hardware_registry = Default::default();

        let (ecat_controller, mut sub_devices) =
            if let EtherCATMode::Enabled(config) = config.ethercat_mode {
                ethercat::init::<B>(config, &mut bootstrap, &mut hardware_registry)?
            } else {
                (None, Default::default())
            };

        // --- initialize modbus rtu ---
        // TODO: title says it ...

        // --- build machines ---
        bootstrap.submit_event(RuntimeInitEvent::BuildingMachines)?;

        let mut resources = Resources::default();
        let machines = Self::init_machines(
            &mut bootstrap,
            &machine_registry,
            &hardware_registry,
            ecat_controller.as_ref().map(|v| v.channel.clone()),
            &mut resources,
        )?;

        // --- finalize ethercat ---
        bootstrap.submit_event(RuntimeInitEvent::EtherCATFinalizing)?;

        if let Some(controller) = &ecat_controller {
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
            bridge: bootstrap.finish(),
            last_export_ts: Instant::now(),
            subscriptions: Default::default(),
        })
    }

    fn init_machines(
        bridge: &mut B::Bootstrap,
        machine_registry: &MachineRegistry,
        hardware_registry: &HardwareRegistry,
        ecat_interface: Option<EtherCATThreadChannel>,
        resources: &mut Resources,
    ) -> RuntimeInitializeResult<Vec<MachineInstance>> {
        let mut machines: Vec<MachineInstance> = Vec::new();

        for (ident_unique, hardware) in hardware_registry {
            let ident = ident_unique.identification;

            let Some(build) = machine_registry.get(&ident) else {
                todo!()
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
