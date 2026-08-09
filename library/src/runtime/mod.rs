use std::cell::Cell;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Instant;

use bitvec::order::Lsb0;
use bitvec::slice::BitSlice;
use chrono::Utc;
use qitech_framework_core::report::CommandEvent;
use qitech_framework_core::report::CommandRecord;
use qitech_framework_core::report::MeasurementSnapshot;
use qitech_framework_core::report::RuntimeEvent;
use qitech_framework_core::report::RuntimeReport;
use qitech_framework_core::report::error::ActErrorImpact;
use qitech_framework_core::session::RuntimeTransport;
use qitech_framework_core::session::runtime::SessionRunning;
use types::Config;

pub mod error;

mod types;
pub use types::EtherCATController;
pub use types::EtherCATSubDevice;
pub use types::MachineRegistry;

mod ethercat;
mod init;
mod modbus_rtu;
mod utils;

mod config;
pub use config::EtherCATConfig;
pub use config::RuntimeConfiguration;

use crate::journal::Journals;
use crate::machine::MachineInstance;
use crate::machine::ResourceRegistry;
use crate::runtime::error::RuntimeError;
mod request;

pub struct Runtime<T: RuntimeTransport> {
    report: RuntimeReport,
    session: SessionRunning<T>,

    // --- resource managers ---
    journals: Journals,
    resources: ResourceRegistry,

    // --- instances ---
    machines: Vec<MachineInstance>,
    sub_devices: Vec<EtherCATSubDevice>,
    ecat_controller: Option<EtherCATController>,

    // --- misc ---
    config: Config,
    last_export_ts: Instant,

    /// how many reports have we exported
    export_count: Rc<Cell<u64>>,
}

impl<T: RuntimeTransport> Runtime<T> {
    pub fn run(mut self) -> Result<(), RuntimeError> {
        loop {
            let now = Instant::now();
            self.tick(now)?;
        }
    }

    fn tick(&mut self, now: Instant) -> Result<(), RuntimeError> {
        if self.controller_finished() {
            return Err(RuntimeError::EtherCATControllerDied);
        }

        self.write_ecat_inputs();
        self.process_requests();
        self.run_machines(now);

        // --- sync cache so subscribed properties get the latest data next cycle ---
        self.resources.config_properties.sync_cache();
        self.resources.state_properties.sync_cache();
        self.resources.measurements.sync_cache();

        // self.resources.sync_caches();
        self.write_ecat_outputs();
        self.export_report_if_due(now);

        let elapsed = now.elapsed();
        if let Some(remaining) = self.config.cycle_timeout.checked_sub(elapsed) {
            sleep(remaining);
        } else {
            // cycle overran its budget
        }

        Ok(())
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

        // --- extract config property events ---
        self.journals.config_property.drain_with(|x| {
            self.report.machines.config_property_records.push(x);
        });

        // --- extract state property events ---
        self.journals.state_property.drain_with(|x| {
            self.report.machines.state_property_records.push(x);
        });

        // --- sample measurements ---
        for descriptor in self.resources.measurements.iter() {
            let convert = descriptor.metadata;
            let value = unsafe { (convert)(descriptor.p_value) };

            // TODO: faster way to eliminate slots !
            if !self.machines.iter().any(|x| x.ident == descriptor.ident) {
                // machine is disabled, skip
                continue;
            }

            self.report
                .machines
                .measurement_snapshots
                .push(MeasurementSnapshot {
                    machine: descriptor.ident,
                    path: descriptor.resource.to_string(),
                    value,
                });
        }

        // --- scan for capability updates ---
        let journal = self.journals.commands.new_handle();

        for instance in &mut self.machines {
            for (path, handle) in &mut instance.commands {
                if let Some(get_capability) = &handle.can_execute_fn {
                    let capability = (get_capability)(instance.machine.as_ref());

                    if capability != handle.capability_prev {
                        journal.append(CommandRecord {
                            timestamp: Utc::now(),
                            machine: instance.ident,
                            path: path.to_string(),
                            event: CommandEvent::CapabilityChanged(capability.clone()),
                        });
                    }

                    handle.capability_prev = capability;
                }
            }
        }

        self.journals.commands.drain_with(|x| {
            self.report.machines.command_records.push(x);
        });

        // --- export report ---
        self.session.send_report(self.report.clone()).unwrap();

        // --- reset report data ---
        self.report.reset();

        // --- reset timer ---
        self.last_export_ts = now;
        self.export_count.set(self.export_count.get() + 1);
    }

    fn run_machines(&mut self, now: Instant) {
        let mut i = 0;

        while i < self.machines.len() {
            match self.machines[i].machine.act(now) {
                Ok(()) => i += 1,

                Err(e) if e.impact != ActErrorImpact::Irrecoverable => i += 1,

                Err(_) => {
                    // --- machine cannot recover, remove it ---
                    let MachineInstance { ident, .. } = self.machines.swap_remove(i);

                    // --- free up resources ---
                    // self.resources.clear_machine(ident);

                    // --- record the change ---
                    self.report
                        .events
                        .push(RuntimeEvent::RemovedMachine { ident });
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
