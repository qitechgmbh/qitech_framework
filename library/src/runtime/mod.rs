use std::cell::RefCell;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

use bitvec::order::Lsb0;
use bitvec::slice::BitSlice;
use qitech_framework_common::MachineIdentificationUnique;
use qitech_lib::ethercat_hal::MetaSubdevice;
use qitech_lib::ethercat_hal::devices::EthercatDevice;

use crate::machine::BuildContext;
use crate::machine::Machine;
use crate::machine::Resources;
use crate::runtime::init::RuntimeInitializer;
use crate::runtime::types::HardwareRegistry;

mod types;
pub use types::EtherCATController;
pub use types::EtherCATSubDevice;
pub use types::MachineRegistry;

mod init;
pub use init::EtherCATConfig;

pub struct Runtime {
    // --- registries ---
    machine_registry: MachineRegistry,
    hardware_registry: HardwareRegistry,

    // --- resource managers ---
    resources: Resources,

    // --- connections ---
    // session: Session with hub
    ecat_controller: Option<EtherCATController>,

    // --- instances ---
    machines: Vec<(MachineIdentificationUnique, Box<dyn Machine>)>,
    sub_devices: Vec<(MetaSubdevice, Rc<RefCell<dyn EthercatDevice + 'static>>)>,
}

impl Runtime {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> RuntimeInitializer {
        RuntimeInitializer::new()
    }

    pub fn run(mut self) -> Result<(), &'static str> {
        loop {
            // TODO: exit if controller is finished
            self.write_ecat_inputs();

            // TODO: receive data from hub
            // TODO: process connection requests
            // TODO: run config mutations
            // TODO: run commands

            self.execute_machines();
            self.write_ecat_outputs();

            // TODO: export data to hub

            sleep(Duration::from_micros(100));
        }
    }

    fn build_machines(&mut self) {
        for (ident_unique, hardware) in &self.hardware_registry {
            let ident = ident_unique.identification;

            let Some(build) = self.machine_registry.get(&ident) else {
                todo!()
                // bail!("Failed to find registry entry for machine {{{ident}}}");
            };

            let ethercat_interface = self.ecat_controller.as_ref().map(|v| v.channel.clone());

            let ctx = BuildContext::new(
                *ident_unique,
                ethercat_interface,
                &mut self.resources,
                hardware.clone(),
            );

            println!("Building machine `{ident_unique}`");

            let machine = match (build)(ctx) {
                Ok(v) => v,
                Err(e) => {
                    println!("Failed to build machine: {e}");
                    continue;
                }
            };

            self.machines.push((*ident_unique, machine));
        }
    }

    fn process_subscribe(
        &mut self,
        ident_source: MachineIdentificationUnique,
        ident_subscriber: MachineIdentificationUnique,
    ) {
        let Some((_, source)) = self
            .machines
            .iter()
            .find(|(ident, _)| *ident == ident_source)
        else {
            return;
        };

        let Some((_, subscriber)) = self
            .machines
            .iter()
            .find(|(ident, _)| *ident == ident_subscriber)
        else {
            return;
        };

        // subscriber.subscribe(ctx):
    }

    fn execute_machines(&mut self) {
        // --- act pass ---
        for (_, machine) in &mut self.machines {
            let res = machine.act();
            _ = res; // TODO: use error
        }

        // --- react pass ---
        for (_, machine) in &mut self.machines {
            // let res = machine.react();
            // _ = res; // TODO: use error
        }
    }

    fn write_ecat_inputs(&mut self) {
        let Some(controller) = &mut self.ecat_controller else {
            return;
        };

        let inputs = controller
            .app_handle
            .get_inputs()
            .expect("There should always be an input (latest state)");

        for i in 0..self.sub_devices.len() {
            let (meta_dev, dev) = &self.sub_devices[i];

            let input_slice = &inputs[meta_dev.start_tx..meta_dev.end_tx];
            let input_bits_slice = BitSlice::<u8, Lsb0>::from_slice(input_slice);

            // why are we ignoring these errors ?
            let mut dev = dev.borrow_mut();
            _ = dev.input(input_bits_slice);
            _ = dev.input_post_process();
        }
    }

    fn write_ecat_outputs(&mut self) {
        let Some(controller) = &mut self.ecat_controller else {
            return;
        };

        let Some(outputs) = controller.app_handle.write_outputs() else {
            return;
        };

        for i in 0..self.sub_devices.len() {
            let (meta_dev, dev) = &self.sub_devices[i];

            let output_slice = &mut outputs[meta_dev.start_rx..meta_dev.end_rx];
            let output_bits = BitSlice::<u8, Lsb0>::from_slice_mut(output_slice);

            // why are we ignoring these errors ?
            let mut dev = dev.borrow_mut();
            _ = dev.output_pre_process();
            _ = dev.output(output_bits);
        }

        controller.app_handle.send_outputs();
    }
}
