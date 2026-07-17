use std::cell::RefCell;
use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;
use anyhow::bail;
use bitvec::order::Lsb0;
use bitvec::slice::BitSlice;
use control_core::{MachineIdentification, MachineIdentificationUnique};
use qitech_lib::ethercat_hal::MetaSubdevice;
use qitech_lib::ethercat_hal::devices::EthercatDevice;

use crate::data::DataStore;
use crate::machine_registry::MachineRegistryEntry;
use crate::{Config, Machine, MachineBuilder, MachineRegistry, ethercat};
use crate::machine::MachineHardwareRegistry;

pub struct Runtime {
    registry: MachineRegistry,
    machines: Vec<(MachineIdentificationUnique, Box<dyn Machine>)>,
    hardware_registry: MachineHardwareRegistry,
    controller: Option<ethercat::Controller>,
    data_store: DataStore,
    sub_devices: Vec<(MetaSubdevice, Rc<RefCell<dyn EthercatDevice + 'static>>)>,
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
        let sub_devices = ethercat::setup(&controller, &mut hardware)?;

        let mut runtime = Self {
            registry,
            machines: vec![],
            hardware_registry: hardware,
            controller: Some(controller),
            data_store: DataStore::new(),
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
            // TODO: exit if controller is finished
            self.write_ecat_inputs()?;
            self.execute_machines()?;
            self.write_ecat_outputs();

            sleep(Duration::from_micros(100));
        }
    }

    pub fn build_machines(&mut self) -> anyhow::Result<()> {
        for (ident_unique, hardware) in &self.hardware_registry {
            let ident = MachineIdentification::from(*ident_unique);

            let Some(MachineRegistryEntry { build, schema }) = self.registry.find(ident) else {
                bail!("Failed to find registry entry for machine {{{ident}}}");
            };

            let ethercat_interface = self.controller.as_ref().map(|v| v.channel.clone());

            let builder = MachineBuilder::new(
                *ident_unique, 
                hardware.clone(), 
                ethercat_interface,
                &mut self.data_store
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

            let mut dev = dev.borrow_mut();
            _ = dev.output_pre_process();
            _ = dev.output(output_bits);
        }

        controller.app_handle.send_outputs();
    }
}
