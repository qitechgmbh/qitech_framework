use std::collections::HashMap;
use std::thread::sleep;
use std::time::Instant;

use bitvec::order::Lsb0;
use bitvec::slice::BitSlice;
use chrono::Utc;
use qitech_framework_common::MachineIdentificationUnique;
use qitech_framework_common::RuntimeReport;
use qitech_framework_common::link::RuntimeTransport;
use qitech_framework_common::link::runtime::session;
use types::Config;
use types::MachineInstance;

use crate::machine::Resources;

pub mod error;

mod types;
pub use types::EtherCATController;
pub use types::EtherCATSubDevice;
pub use types::MachineRegistry;
pub use types::RuntimeStatus;

mod ethercat;
mod init;
mod modbus_rtu;
mod utils;

mod config;
pub use config::EtherCATConfig;
pub use config::RuntimeConfiguration;

mod request;

pub struct Runtime<T: RuntimeTransport> {
    status: RuntimeStatus,

    // // --- registries ---
    // machine_registry: MachineRegistry,
    // hardware_registry: HardwareRegistry,

    // --- resource managers ---
    resources: Box<Resources>,
    report: RuntimeReport,

    // --- instances ---
    machines: Vec<MachineInstance>,
    sub_devices: Vec<EtherCATSubDevice>,
    subscriptions: HashMap<MachineIdentificationUnique, Vec<MachineIdentificationUnique>>,

    // --- misc ---
    ecat_controller: Option<EtherCATController>,
    config: Config,
    session: session::Running<T>,
    last_export_ts: Instant,
}

impl<T: RuntimeTransport> Runtime<T> {
    pub fn run(mut self) {
        loop {
            let now = Instant::now();
            self.tick(now);

            if self.tick(now) != RuntimeStatus::Running {
                break;
            }
        }
    }

    pub fn tick(&mut self, now: Instant) -> RuntimeStatus {
        if self.status == RuntimeStatus::Stopped {
            return self.status;
        }

        if self.controller_finished() {
            self.status = RuntimeStatus::Stopped;
            return self.status;
        }

        self.write_ecat_inputs();
        self.process_requests();
        self.run_machines();
        self.resources.sync_caches();
        self.write_ecat_outputs();
        self.export_report_if_due(now);

        let elapsed = now.elapsed();
        if let Some(remaining) = self.config.cycle_timeout.checked_sub(elapsed) {
            sleep(remaining);
        } else {
            // cycle overran its budget
        }

        RuntimeStatus::Running
    }

    fn controller_finished(&self) -> bool {
        self.ecat_controller
            .as_ref()
            .and_then(|c| c.join_handle.as_ref())
            .is_some_and(|h| h.is_finished())
    }

    fn export_report_if_due(&mut self, now: Instant) {
        // --- check if export is due ---
        if now.duration_since(self.last_export_ts) < self.config.export_interval {
            return;
        }

        // --- collect data ---
        self.report.timestamp = Utc::now();
        self.resources.extract_report(&mut self.report.machines);

        // --- export report ---
        self.session.send_report(self.report.clone()).unwrap();

        // --- clear buffers ---
        self.report.logs.clear();
        self.report.responses.clear();
        self.report.machines.config_mutations.clear();
        self.report.machines.state_mutations.clear();
        self.report.machines.measurements.clear();
        self.report.machines.events.clear();
        self.report.machines.commands.clear();

        // --- reset timer ---
        self.last_export_ts = now;
    }

    fn run_machines(&mut self) {
        let mut i = 0;

        while i < self.machines.len() {
            match self.machines[i].inner.act() {
                Ok(()) => i += 1,

                Err(e) if e.recoverable => i += 1,

                Err(_) => {
                    // --- machine cannot recover, remove it ---
                    let MachineInstance { ident, .. } = self.machines.swap_remove(i);

                    // --- free up resources ---
                    self.resources.clear_machine(ident);

                    // TODO: handle/log error
                }
            }
        }
    }

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
