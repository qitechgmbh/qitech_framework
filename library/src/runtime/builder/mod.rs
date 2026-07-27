use std::time::Duration;
use std::time::Instant;

use qitech_framework_common::MachineSchema;
use qitech_framework_common::RuntimeInitEvent;
use qitech_framework_common::RuntimeReport;
use qitech_lib::ethercat_hal::EtherCATThreadChannel;

use crate::machine::BuildContext;
use crate::machine::Machine;
use crate::machine::MachineBuild;
use crate::machine::MachineInterface;
use crate::machine::Resources;
use crate::machine::error::BuildResult;
use crate::runtime::MachineRegistry;
use crate::runtime::Runtime;
use crate::runtime::bridge::BridgeInitializer;
use crate::runtime::builder::error::RuntimeInitializeError;
use crate::runtime::builder::error::RuntimeInitializeResult;
use crate::runtime::types::BuildMachineFn;
use crate::runtime::types::Config;
use crate::runtime::types::HardwareRegistry;
use crate::runtime::types::MachineInstance;

mod error;
mod ethercat;
mod serial;

mod types;
pub use types::EtherCATConfig;
use types::EtherCATMode;
use types::ModbusMode;

pub struct RuntimeBuilder {
    config: Config,

    machines: Vec<(&'static str, BuildMachineFn)>,
    ethercat_mode: EtherCATMode,
    modbus_rtu_mode: ModbusMode,
}

impl RuntimeBuilder {
    pub fn new() -> Self {
        Self {
            config: Default::default(),
            machines: Default::default(),
            ethercat_mode: EtherCATMode::Disabled,
            modbus_rtu_mode: ModbusMode::Disabled,
        }
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

    // pub fn with_ethercat_mock(mut self) -> Self {
    //     self.ethercat_mode = EtherCATMode::Mock;
    //     self
    // }

    pub fn with_modbus_rtu(mut self) -> Self {
        self.modbus_rtu_mode = ModbusMode::Enabled;
        self
    }

    pub fn machine<M>(mut self) -> Self
    where
        M: Machine + MachineBuild + MachineInterface + 'static,
    {
        fn build_adapter<T>(builder: BuildContext<'_>) -> BuildResult<Box<dyn Machine>>
        where
            T: MachineBuild + Machine + 'static,
        {
            Ok(Box::new(T::build(builder)?))
        }

        self.machines.push((M::SCHEMA, build_adapter::<M>));
        self
    }

    /// attempts to create a new runtime with the provided configuration
    pub fn build<B: BridgeInitializer>(
        self,
        mut bridge: B,
    ) -> RuntimeInitializeResult<Runtime<B::Output>> {
        // --- send hello ---
        bridge.send_hello()?;

        // --- create machine registry ---
        let mut machine_registry = MachineRegistry::default();

        for (schema_str, build_fn) in self.machines {
            let schema = MachineSchema::from_yaml_str(schema_str)?;

            if machine_registry
                .insert(schema.identification, build_fn)
                .is_some()
            {
                return Err(RuntimeInitializeError::DuplicateMachine(
                    schema.identification,
                ));
            }

            bridge.sync_machine(schema_str)?;
        }

        // --- initialize hardware ---
        let mut hardware_registry = Default::default();

        let (ecat_controller, mut sub_devices) =
            if let EtherCATMode::Enabled(config) = self.ethercat_mode {
                ethercat::init(&mut bridge, config, &mut hardware_registry)?
            } else {
                (None, Default::default())
            };

        // --- build machines ---
        let mut resources = Resources::default();
        let mut machines = Vec::new();

        bridge.submit_event(RuntimeInitEvent::BuildingMachines)?;

        build_machines(
            &mut bridge,
            &machine_registry,
            &hardware_registry,
            ecat_controller.as_ref().map(|c| c.channel.clone()),
            &mut resources,
            &mut machines,
        )?;

        bridge.submit_event(RuntimeInitEvent::EtherCATFinalizing)?;
        if let Some(controller) = &ecat_controller {
            ethercat::finalize(controller, &mut sub_devices)?;
        }

        // --- create runtime ---
        Ok(Runtime {
            config: self.config,
            machine_registry,
            hardware_registry,
            resources,
            report: RuntimeReport::default(),
            ecat_controller,
            machines,
            sub_devices,
            subscriptions: Default::default(),
            last_export_ts: Instant::now(),
            bridge: bridge.upgrade(),
        })
    }
}

impl Default for RuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// --- utils ---
pub fn build_machines<B: BridgeInitializer>(
    bride: &mut B,
    machine_registry: &MachineRegistry,
    hardware_registry: &HardwareRegistry,
    ecat_interface: Option<EtherCATThreadChannel>,
    resources: &mut Resources,
    machines: &mut Vec<MachineInstance>,
) -> RuntimeInitializeResult<()> {
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

        bride.submit_event(RuntimeInitEvent::BuiltMachine {
            ident: *ident_unique,
        })?;
    }

    Ok(())
}
