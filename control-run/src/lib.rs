use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use control_core::MachineIdentification;
use control_core::MachineIdentificationUnique;
use control_core::schema::latest::MachineSchema;

mod data;
use data::DataStore;
use data::DataRegistry;

mod serial;

mod ethercat;

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
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

// pub use machine::Command;

pub struct Config {
    pub stay_in_preop: bool,
    pub force_eth_up: bool,
    pub serial_scan_interval: Duration,
    pub interface_discovery_retry_interval: Duration,
}   

// hub keeps track of last connection and can accept it back without full init process
// hub stage one: listen only, stage two: activate -> goto processing, allows to notify frontend

pub fn run(
    rt: Runtime,
    // hub: ControlHubSession,
    config: Config,
    registry: MachineRegistry
) -> anyhow::Result<()> {
    // --- step one: connect to hub for event transmission --- 
    
    // TODO: implement

    // --- step two: ethercat setup ---

    // hub.notify(Discovery start);

    // discover hardware, write down which hardware relates to which machine, cannot be changed once in place
    // hotplug works 100% on that assumption

    let interface = ethercat::find_interface(config.interface_discovery_retry_interval);

    // hub.notify(Discovery complete);

    let eth_control = ethercat::init(&interface);

    // hub.notify(Discovery complete);

    // --- step three: serial stuff ---
    for port_info in serialport::available_ports()? {
        // compare against registry and install hooks

    }

    serialport::available_ports();

    let serial_ports = tokio_serial::available_ports();

    // start scanning for serial ports
    let serial_scanner_task = rt.spawn(serial::run_scanner(
        config.serial_scan_interval, 
        serial_ports_tx,
    ));

    // --- step four: setup ethercat --- 
    ethercat::setup(&eth_control)?;

    // hub.notify(EtherCatState);
    // hub.notify(ShowEtherCatDevices);

    if config.stay_in_preop {
        // send_setup_done_events(state.clone());

        println!("Staying in PreOp as requested, exiting after setup.");
        loop { thread::sleep(Duration::from_secs(1)); }
    }

    // --- step five: build machines --- 




    // ...

    // by dropping the channel the scan should exit gracefully
    drop(serial_ports_rx);

    let res = rt.block_on(async {
        tokio::join!(serial_scanner_task)
    });

    _ = res;

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

        if now.duration_since(last_check) >= config.serial_scan_interval {
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

fn monitor_serial_ports() -> anyhow::Result<()> {
    let monitor = udev::MonitorBuilder::new()?
        .match_subsystem("tty")?
        .listen()?;

    for event in monitor.iter() {
        let serial_ports = serialport::available_ports();
    }

    Ok(())
}

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