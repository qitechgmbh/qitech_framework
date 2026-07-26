use std::collections::HashMap;
use std::thread::sleep;
use std::time::Instant;

use bitvec::order::Lsb0;
use bitvec::slice::BitSlice;
use chrono::Utc;
use qitech_framework_common::MachineIdentificationUnique;
use qitech_framework_common::RuntimeReport;

use crate::machine::BuildContext;
use crate::machine::Machine;
use crate::machine::SyncContext;
use crate::machine::Resources;
use crate::machine::error::ActResult;
use crate::runtime::init::RuntimeBuilder;
use crate::runtime::types::Config;
use crate::runtime::types::HardwareRegistry;
use crate::runtime::types::MachineInstance;

mod types;
pub use types::EtherCATController;
pub use types::EtherCATSubDevice;
pub use types::MachineRegistry;

mod utils;

mod init;
pub use init::EtherCATConfig;

mod request;

// mod utils;
// use utils::build_machines;

pub struct Runtime {
    // --- registries ---
    machine_registry: MachineRegistry,
    hardware_registry: HardwareRegistry,

    // --- resource managers ---
    resources: Resources,
    report: RuntimeReport,

    // --- connections ---
    // session: Session with hub
    ecat_controller: Option<EtherCATController>,

    // --- instances ---
    machines: Vec<MachineInstance>,
    sub_devices: Vec<EtherCATSubDevice>,

    // --- misc ---
    config: Config,
    last_export_ts: Instant,
    subscriptions: HashMap<MachineIdentificationUnique, Vec<MachineIdentificationUnique>>,
}

impl Runtime {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> RuntimeBuilder {
        RuntimeBuilder::new()
    }

    pub fn run(mut self) -> Result<(), &'static str> {
        loop {
            let now = Instant::now();

            // TODO: exit if controller is finished
            self.write_ecat_inputs();

            // TODO: receive data from hub
            // TODO: process connection requests
            // TODO: run config mutations
            // TODO: run commands
            // TODO: laser hotplug

            self.run_machines();
            self.write_ecat_outputs();

            if now.duration_since(self.last_export_ts) >= self.config.export_interval {
                self.export_report();
            }

            sleep(self.config.cycle_timeout);
        }
    }

    // --- hub management ---
    fn export_report(&mut self) {
        // --- collect data ---
        self.report.timestamp = Utc::now();
        self.resources.init_report(&mut self.report.machines);

        // TODO: send to hub

        // --- reset buffers ---
        self.report.logs.clear();
        self.report.responses.clear();
        self.report.runtime.events.clear();
        self.report.runtime.state_mutations.clear();
    }

    // --- machine managment ---
    fn build_machines(&mut self) {
        for (ident_unique, hardware) in &self.hardware_registry {
            let ident = ident_unique.identification;

            let Some(build) = self.machine_registry.get(&ident) else {
                todo!()
                // bail!("Failed to find registry entry for machine {{{ident}}}");
            };

            let ecat_interface = self.ecat_controller.as_ref().map(|v| v.channel.clone());

            let ctx = BuildContext::new(
                *ident_unique,
                ecat_interface,
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

    fn run_machines(&mut self) {
        // let ctx = SyncContext::new(&self.resources);
        // Self::run_machines_pass(&mut self.resources, &mut self.machines, |m| m.act());
        // Self::run_machines_pass(&mut self.resources, &mut self.machines, |m| m.react(&ctx));
    }

    /* 
    fn run_machines_pass(
        resources: &mut Resources,
        machines: &mut Vec<MachineInstance>,
        mut step: impl FnMut(&mut dyn Machine) -> ActResult,
    ) {
        let mut i = 0;
        while i < machines.len() {
            let (ident, machine) = &mut machines[i];
            match step(machine.as_mut()) {
                Ok(()) => i += 1,
                Err(e) if e.recoverable => i += 1,
                Err(_) => {
                    // machine cannot recover from this error.
                    // remove using swap and pop, meaning we don't increment. 
                    machines.swap_remove(i);

                    // free up resources
                    resources.clear_machine(*ident);

                    // TODO: handle/log error
                }
            }
        }
    }
    */

    // --- ethercat managment ---
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
