use std::cell::Cell;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

use bitvec::order::Lsb0;
use bitvec::slice::BitSlice;
use chrono::Utc;
use qitech_framework_core::ident::MachineInstanceIdentification;
use qitech_framework_core::report::CommandEvent;
use qitech_framework_core::report::MeasurementSnapshot;
use qitech_framework_core::report::RuntimeEvent;
use qitech_framework_core::report::RuntimeReport;
use qitech_framework_core::report::error::ActErrorImpact;
use qitech_framework_core::session::RuntimeTransport;
use qitech_framework_core::session::runtime::SessionRunning;
use qitech_lib::xtrem::XtremBusHandle;
use types::Config;

pub mod error;

mod types;
use types::EtherCATController;
use types::EtherCATSubDevice;
use types::HardwareRegistry;
use types::MachineInstance;
use types::MachineRegistry;

mod ethercat;
mod init;
mod modbus_rtu;
mod utils;
mod xtrem;

mod config;
pub use config::EtherCATConfig;
use config::ModbusRtuConfig;
pub use config::RuntimeConfiguration;
pub use config::XtremConfig;
pub use xtrem::XtremDeviceBuild;

use crate::machine::Hardware;
use crate::machine::hardware::ModbusRTUDeviceIdentified;
use crate::resource::Journals;
use crate::resource::ResourceRegistry;
use crate::runtime::error::RuntimeError;
mod request;

pub struct Runtime<T: RuntimeTransport> {
    report: RuntimeReport,
    session: SessionRunning<T>,

    // --- resource managers ---
    journals: Journals,
    resources: ResourceRegistry,

    // --- registries ---
    machine_registry: MachineRegistry,
    hardware_registry: HardwareRegistry,

    // --- instances ---
    machine_instances: Vec<MachineInstance>,
    sub_devices: Vec<EtherCATSubDevice>,
    ecat_controller: Option<EtherCATController>,

    _xtrem_bus: Option<XtremBusHandle>,
    modbus: Option<modbus_rtu::ModbusManager>,
    modbus_config: Option<ModbusRtuConfig>,

    // --- misc ---
    config: Config,
    last_export_ts: Instant,

    /// how many reports have we exported
    export_count: Rc<Cell<u64>>,

    config_mode: bool,
}

impl<T: RuntimeTransport> Runtime<T> {
    pub fn run(mut self) -> Result<(), RuntimeError> {
        let mut last_update = Instant::now();

        loop {
            let now = Instant::now();
            let dt = now.duration_since(last_update);
            last_update = now;
            self.tick(now, dt)?;
        }
    }

    fn tick(&mut self, now: Instant, dt: Duration) -> Result<(), RuntimeError> {
        if self.controller_finished() {
            return Err(RuntimeError::EtherCATControllerDied);
        }

        if !self.config_mode {
            self.write_ecat_inputs();
        }

        self.process_requests();

        if !self.config_mode {
            self.update_modbus();
            self.run_machines(dt);
        }

        // --- sync cache so subscribed properties get the latest data next cycle ---
        self.resources.config_properties.sync_cache();
        self.resources.state_properties.sync_cache();
        self.resources.measurements.sync_cache();

        if !self.config_mode {
            self.write_ecat_outputs();
        }

        // --- record timings ---
        self.report
            .timings
            .record(now.elapsed(), self.config.cycle_period);

        // --- export if due ---
        self.export_report_if_due(now);

        // --- sleep remaining duration ---
        let elapsed = now.elapsed();
        if let Some(remaining) = self.config.cycle_period.checked_sub(elapsed) {
            sleep(remaining);
        }

        Ok(())
    }

    fn update_modbus(&mut self) {
        let Some(modbus) = &mut self.modbus else {
            return;
        };

        let mut events = Vec::new();
        modbus.update(|ident, event| events.push((ident, event.clone())));

        let mut hardware_added = false;

        for (ident, event) in events {
            match event {
                modbus_rtu::WatchEvent::Detached { binding, device } => {
                    tracing::warn!(%ident, binding, device, "modbus rtu device detached");

                    if self.remove_modbus_hardware(ident, &binding) {
                        self.remove_machine(ident);
                    }
                }

                modbus_rtu::WatchEvent::Attached { binding, device } => {
                    tracing::info!(%ident, binding, device, "modbus rtu device attached");

                    // --- drop a stale device still bound to this path ---
                    if self.remove_modbus_hardware(ident, &binding) {
                        self.remove_machine(ident);
                    }

                    let Some(entry) = self
                        .modbus_config
                        .as_ref()
                        .and_then(|config| config.entries.get(&binding))
                    else {
                        continue;
                    };

                    let device = match (entry.init)(device) {
                        Ok(v) => v,
                        Err(e) => {
                            tracing::error!(%ident, binding, e, "modbus rtu device init failed");
                            continue;
                        }
                    };

                    self.hardware_registry
                        .entry(ident)
                        .or_default()
                        .push(Hardware::ModbusRTU(ModbusRTUDeviceIdentified {
                            device,
                            binding,
                        }));

                    hardware_added = true;
                }
            }
        }

        if hardware_added {
            self.build_machines();
        }
    }

    /// Removes the modbus rtu hardware bound to `binding` from `ident`, returns if any was removed
    fn remove_modbus_hardware(&mut self, ident: MachineInstanceIdentification, binding: &str) -> bool {
        let Some(hardware) = self.hardware_registry.get_mut(&ident) else {
            return false;
        };

        let len = hardware.len();
        hardware.retain(|hw| !matches!(hw, Hardware::ModbusRTU(m) if m.binding == binding));
        let removed = hardware.len() != len;

        if hardware.is_empty() {
            self.hardware_registry.remove(&ident);
        }

        removed
    }

    fn remove_machine(&mut self, ident: MachineInstanceIdentification) {
        let Some(i) = self.machine_instances.iter().position(|m| m.ident == ident) else {
            return;
        };

        self.machine_instances.swap_remove(i);
        self.report
            .events
            .push(RuntimeEvent::RemovedMachine { ident });
    }

    /// Builds every machine in the hardware registry that has no instance yet
    fn build_machines(&mut self) {
        let results = utils::build_machines(
            self.export_count.clone(),
            &self.machine_registry,
            &self.hardware_registry,
            self.ecat_controller.as_ref().map(|c| c.channel.clone()),
            &mut self.journals,
            &mut self.resources,
            &mut self.machine_instances,
        );

        for (ident, result) in results {
            match result {
                Ok(()) => self.report.events.push(RuntimeEvent::AddedMachine { ident }),
                Err(e) => tracing::warn!(%ident, %e, "failed to build machine"),
            }
        }
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
            if !self.machine_instances.iter().any(|x| x.ident == descriptor.ident) {
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
        for instance in &mut self.machine_instances {
            for (path, handle) in &mut instance.commands {
                if let Some(get_capability) = &handle.can_execute_fn {
                    let capability = (get_capability)(instance.machine.as_ref());

                    if capability != handle.capability_prev {
                        self.journals.command.record(
                            instance.ident,
                            path,
                            CommandEvent::CapabilityChanged(capability.clone()),
                        );
                    }

                    handle.capability_prev = capability;
                }
            }
        }

        self.journals.command.drain_with(|x| {
            self.report.machines.command_records.push(x);
        });

        // --- collect emitted events ---
        self.journals.event.drain_with(|x| {
            self.report.machines.event_records.push(x);
        });

        // --- export report ---
        self.session.send_report(self.report.clone()).unwrap();

        // --- reset report data ---
        self.report.reset();

        // --- reset timer ---
        self.last_export_ts = now;
        self.export_count.set(self.export_count.get() + 1);
    }

    fn run_machines(&mut self, dt: Duration) {
        let mut i = 0;

        while i < self.machine_instances.len() {
            match self.machine_instances[i].machine.act(dt) {
                Ok(()) => i += 1,

                Err(e) if e.impact != ActErrorImpact::Irrecoverable => i += 1,

                Err(_) => {
                    // --- machine cannot recover, remove it ---
                    let MachineInstance { ident, .. } = self.machine_instances.swap_remove(i);

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

            let mut dev = dev.borrow_mut();
            _ = dev.output_pre_process();
            _ = dev.output(output_bits);
        }

        controller.app_handle.send_outputs();
    }
}
