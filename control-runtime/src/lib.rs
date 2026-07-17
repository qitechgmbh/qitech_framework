use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use anyhow::bail;
use control_core::MachineIdentification;
use control_core::MachineIdentificationUnique;
use control_core::schema;
use control_core::schema::latest::MachineSchema;

mod data;
use data::DataStore;
use data::DataRegistry;

// mod state;
// use state::main::MainState;

mod machine;
pub use machine::Machine;
pub use machine::MachineBuild;
pub use machine::MachineBuilder;
pub use machine::MachineBuildError;
pub use machine::MachineActResult;
pub use machine::MachineActError;
pub use machine::ConfigProperty;
pub use machine::StateProperty;
pub use machine::Measurement;
use qitech_lib::ethercat_hal;
use qitech_lib::ethercat_hal::DcConfiguration;
use qitech_lib::ethercat_hal::MasterConfiguration;
use qitech_lib::ethercat_hal::RtOptimizationConfig;

use crate::data::DataRecorder;
use crate::machine::MachineHardwareRegistry;

mod ethercat;

// pub use machine::Command;

#[derive(Debug, Clone)]
pub struct Config {
    pub interface_discovery_retry_interval: Duration,
    pub ethercat: Option<MasterConfiguration>,
}

// hub keeps track of last connection and can accept it back without full init process
// hub stage one: listen only, stage two: activate -> goto processing, allows to notify frontend

pub struct Runtime {
    registry: MachineRegistry,
    machines: Vec<(MachineIdentificationUnique, Box<dyn Machine>)>,
    hardware: MachineHardwareRegistry,
    controller: Option<ethercat::Controller>,
    data_store: DataStore,
    // session: Session with hub
}

impl Runtime {
    pub fn init(
        config: Config, 
        registry: MachineRegistry
    ) -> anyhow::Result<Self>  {
        let interface = ethercat::find_interface(config.interface_discovery_retry_interval);
        println!("using ethercat interface: {interface}");

        let controller = ethercat::init(&interface, config.ethercat);
        println!("initialized ethercat control");

        let mut hardware = MachineHardwareRegistry::new();

        println!("Initializing ethercat");
        let mut sub_devices = ethercat::setup(&controller, &mut hardware)?;

        let mut runtime = Self {
            registry,
            machines: vec![],
            hardware,
            controller: Some(controller),
            data_store: DataStore::new(),
        };

        println!("Building machines");
        runtime.build_machines()?;

        println!("Finalizing ethercat");
        let controller = runtime.controller.as_ref().expect("must exist");
        ethercat::finalize(controller, &mut sub_devices)?;

        println!("Finished");
        Ok(runtime)
    }

    pub fn run(self) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn build_machines(&mut self) -> anyhow::Result<()> {
        for (ident_unique, hardware) in &self.hardware {
            let ident = MachineIdentification::from(*ident_unique);

            let Some(MachineRegistryEntry { build, .. }) = self.registry.find(ident) else {
                bail!("Failed to find registry entry for machine {{{ident}}}");
            };

            let ethercat_interface = self.controller.as_ref().map(|v| v.channel.clone());

            let builder = MachineBuilder::new(
                *ident_unique, 
                hardware.clone(), 
                ethercat_interface,
                &mut self.data_store
            );

            let machine = (build)(builder);
        }

        Ok(())
    }
}





fn monitor_serial_ports() -> anyhow::Result<()> {
    let monitor = udev::MonitorBuilder::new()?
        .match_subsystem("tty")?
        .listen()?;

    for event in monitor.iter() {
        let serial_ports = serialport::available_ports();
    }

    Ok(())
}

/*
fn detect_and_build_machines(main_state: &mut MainState) {
    let idents: Vec<MachineIdentificationUnique> = main_state
        .machines
        .iter()
        .map(|machine| machine.get_identification())
        .collect();

    for key in main_state.hardware.keys() {
        if idents.contains(key) {
            continue;
        }

        let result = MACHINE_REGISTRY
            .build_machine(key.clone(), main_state.hardware.get(key).unwrap().clone());

        match result {
            Ok(machine) => {
                let _res = state.add_machine_sync(
                    key.clone().into(),
                    None,
                    Some(machine.get_api_sender()),
                );
                main_state.machines.push(machine);
            }
            Err(e) => {
                if !main_state.machine_errors.contains_key(key) {
                    let _res =
                        state.add_machine_sync(key.clone().into(), Some(e.to_string()), None);
                }
                main_state.machine_errors.insert(*key, e.to_string());
            }
        };
    }
}
*/

type MachineFactory =
    fn(MachineBuilder<'_>) -> Result<Box<dyn Machine>, MachineBuildError>;

#[derive(Default)]
pub struct MachineRegistry {
    inner: HashMap<MachineIdentification, MachineRegistryEntry>,
}

impl MachineRegistry {
    pub fn register<T>(&mut self, schema: &'static str) -> anyhow::Result<()>
    where
        T: MachineBuild + Machine + 'static,
    {
        let schema = schema::parse_latest(schema)?;

        self.inner.insert(schema.identification, MachineRegistryEntry {
            schema,
            build: Self::build_adapter::<T>,
        });

        Ok(())
    }
        
    fn build_adapter<T>(
        builder: MachineBuilder<'_>,
    ) -> Result<Box<dyn Machine>, MachineBuildError>
    where
        T: MachineBuild + Machine + 'static,
    {
        Ok(Box::new(T::build(builder)?))
    }

    pub fn find(&self, ident: MachineIdentification) -> Option<&MachineRegistryEntry> {
        self.inner.get(&ident)
    }
}

pub struct MachineRegistryEntry {
    schema: MachineSchema,
    build: MachineFactory,
}

impl MachineRegistryEntry {

}

pub enum MachineOperationResult {
    Success,
    Failure {
        reason: String,
        can_recover: bool,
    }
}