use std::collections::HashMap;
use std::time::Duration;

use control_core::MachineIdentification;
use control_core::schema::latest::MachineSchema;

mod data;
use data::DataStore;
use data::DataRegistry;

mod state;
use state::main::MainState;

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
// pub use machine::Command;

pub struct Config {
    pub stay_in_preop: bool,
    pub force_eth_up: bool,
    pub hotplug_duration: Duration,
}

#[derive(Default)]
pub struct MachineRegistry {
    inner: Vec<MachineRegistryEntry>,
}

impl MachineRegistry {
    pub fn register(
        &mut self,
        schema: MachineSchema, 
        build: fn(MachineBuilder) -> Box<dyn Machine>
    ) {
        // TODO: check for duplicates
        self.inner.push(MachineRegistryEntry { schema, build });
    }
}

pub struct MachineRegistryEntry {
    schema: MachineSchema,
    build: fn(MachineBuilder) -> Box<dyn Machine>,
}

impl MachineRegistryEntry {
    pub fn new() {

    }
}

pub fn run(config: Config, registry: MachineRegistry) {
    let mut main_state = MainState::new();
    _ = registry;

    // let b = MachineBuilder;
    // let x = Box::new((registry.first().unwrap().build)(b));

    // let interface = find_ethercat_interface(&shared_state);
    // let eth_control = optimized_ethercat_init(&interface);

    // let (tx, rx) = tokio::sync::mpsc::channel(2);
    // let (tx_ports, mut rx_ports) = tokio::sync::mpsc::channel(2);
    // detect_serial(rx, tx_ports);

    // // Subdevices are known after PreOp — show them in the frontend now
    // send_ethercat_devices_event(state.clone());

    // if stay_in_preop && eth_control.is_some() {
    //     send_setup_done_events(state.clone());
    //     println!("Staying in PreOp as requested, exiting after setup.");
    //     loop {
    //         std::thread::sleep(core::time::Duration::from_secs(1));
    //     }
    // }

    // // detect_and_build_machines must run in PreOp (machines initialize assuming PreOp)
    // detect_and_build_machines(state.clone(), &mut main_state);

    // // Only emit machines to frontend after OP state is confirmed
    // send_machines_event(state.clone());

    let mut last_check = std::time::Instant::now();

    loop {
        let now = std::time::Instant::now();

        // match &mut eth_control {
        //     Some(control) => {
        //         if control
        //             .join_handle
        //             .as_ref()
        //             .expect("Join handle should be some")
        //             .is_finished()
        //         {
        //             return;
        //         }
        //         write_ecat_inputs(&mut control.app_handle, main_state.subdevices.clone());
        //     }
        //     None => (),
        // };

        // let machines_to_remove =
        //     run_machines(&mut main_state.machines, &mut main_state.machine_data_reg);
        //     
        // if machines_to_remove.is_some() {
        //     remove_machines(&mut main_state, state.clone(), machines_to_remove);
        // }

        if now.duration_since(last_check) >= config.hotplug_duration {
            // let _ = tx.try_send(());
            // let _ = laser_hotplug(&mut main_state, state.clone(), &mut rx_ports);
            last_check = now;
        }

        // match &mut eth_control {
        //     Some(control) => {
        //         write_ecat_outputs(&mut control.app_handle, main_state.subdevices.clone());
        //     }
        //     None => (),
        // };

        std::thread::sleep(Duration::from_micros(100));
    }
}
