use std::cell::RefCell;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;
use bitvec::order::Lsb0;
use bitvec::slice::BitSlice;
use qitech_framework_common::MachineIdentificationUnique;
use qitech_lib::ethercat_hal::MetaSubdevice;
use qitech_lib::ethercat_hal::devices::EthercatDevice;

use crate::machine::{Machine, resource::{CommandManager, ConfigPropertyManager}};

mod ethercat;

mod hardware;
use hardware::MachineHardwareRegistry;

mod machine_registry;
use machine_registry::MachineRegistry;


pub struct Runtime {
    // --- registries ---
    machine_registry: MachineRegistry,
    hardware_registry: MachineHardwareRegistry,

    // --- resources ---
    config_properties: ConfigPropertyManager,
    state_properties: StatePropertyManager,
    measurements: MeasurementManager,
    commands: CommandManager,
    events: EventManager,

    // --- connections
    // session: Session with hub
    controller: Option<ethercat::Controller>,

    // --- instances ---
    machines: Vec<(MachineIdentificationUnique, Box<dyn Machine>)>,
    sub_devices: Vec<(MetaSubdevice, Rc<RefCell<dyn EthercatDevice + 'static>>)>,
}

impl Runtime {
    pub fn init(
        config: Config, 
        registry: MachineRegistry
    ) -> anyhow::Result<Self>  {
        let interface = ethercat::find_interface(config.interface_discovery_retry_interval);
        println!("using ethercat interface: {interface}");

        let controller = ethercat::init_controller(&interface, config.ethercat);
        println!("initialized ethercat control");

        let mut hardware = MachineHardwareRegistry::new();

        println!("Initializing ethercat");
        let sub_devices = ethercat::setup(&controller, &mut hardware)?;

        let mut runtime = Self {
            machine_registry: registry,
            hardware_registry: hardware,
            resource_registry: MachineResourceRegistry::new(),
            resource_journals: ResourceJournals::new(),
            controller: Some(controller),
            machines: vec![],
            sub_devices
        };

        println!("Building machines");
        runtime.build_machines()?;

        println!("Finalizing ethercat");
        let controller = runtime.controller.as_ref().expect("must exist");
        ethercat::finalize(controller, &mut runtime.sub_devices)?;

        Ok(runtime)
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        loop {
            // TODO: receive data from hub

            // TODO: exit if controller is finished
            self.write_ecat_inputs()?;

            // TODO: run config mutations
            // TODO: run commands
            self.execute_machines()?;

            self.write_ecat_outputs();

            // TODO: export data to hub

            sleep(Duration::from_micros(100));
        }
    }

    pub fn build_machines(&mut self) {
        
        for (ident_unique, hardware) in &self.hardware_registry {
            let ident = MachineIdentification::from(*ident_unique);

            let Some(MachineRegistryEntry { build, schema }) = self.machine_registry.find(ident) else {
                bail!("Failed to find registry entry for machine {{{ident}}}");
            };

            let ethercat_interface = self.controller.as_ref().map(|v| v.channel.clone());

            let builder = MachineBuildContext::new(
                *ident_unique, 
                &mut self.resource_registry,
                &mut self.resource_journals,
                ethercat_interface,
                hardware.clone(), 
            );

            println!("Building '{}' with identification '{ident_unique}'", &schema.name);
            let machine = match (build)(builder) {
                Ok(v) => v,
                Err(e) => {
                    println!("Failed to build machine: {e}");
                    continue;
                },
            };

            self.machines.push((*ident_unique, machine));
        }

        Ok(())
    }

    pub fn execute_machines(&mut self) -> anyhow::Result<()> {
        for (_, machine) in &mut self.machines {
            let res = machine.act();
            _ = res; // TODO: use error
        }

        Ok(())
    }

    pub fn write_ecat_inputs(&mut self) -> anyhow::Result<()> {
        let Some(controller) = &mut self.controller else {
            return Ok(());
        };

        let inputs = controller.app_handle
            .get_inputs()
            .expect("There should always be an input (latest state)");

        for i in 0..self.sub_devices.len() {
            let (meta_dev, dev) = &self.sub_devices[i];

            let input_slice = &inputs[meta_dev.start_tx..meta_dev.end_tx];
            let input_bits_slice = BitSlice::<u8, Lsb0>::from_slice(input_slice);

            // why are we ignoring the errors ?
            let mut dev = dev.borrow_mut();
            _ = dev.input(input_bits_slice);
            _ = dev.input_post_process();
        }

        Ok(())
    }

    pub fn write_ecat_outputs(&mut self) {
        let Some(controller) = &mut self.controller else {
            return;
        };

        let Some(outputs) = controller.app_handle.write_outputs() else {
            return;
        };

        for i in 0..self.sub_devices.len() {
            let (meta_dev, dev) = &self.sub_devices[i];

            let output_slice = &mut outputs[meta_dev.start_rx..meta_dev.end_rx];
            let output_bits = BitSlice::<u8, Lsb0>::from_slice_mut(output_slice);

            // why are we ignoring the errors ?
            let mut dev = dev.borrow_mut();
            _ = dev.output_pre_process();
            _ = dev.output(output_bits);
        }

        controller.app_handle.send_outputs();
    }
}
