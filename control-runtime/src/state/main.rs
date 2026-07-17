use std::{cell::RefCell, collections::HashMap, rc::Rc};
use qitech_lib::ethercat_hal::{EtherCATThreadChannel, MetaSubdevice, devices::EthercatDevice, machine_ident_read::MachineDeviceInfo};
use control_core::MachineIdentificationUnique;

use crate::machine::{Hardware, IdentifiedEthercat, Machine, MachineHardware};

pub struct MainState {
    pub subdevices: Vec<(MetaSubdevice, Rc<RefCell<dyn EthercatDevice>>)>,
    pub hardware: HashMap<MachineIdentificationUnique, MachineHardware>,
    pub machines: Vec<Box<dyn Machine>>,
}

impl MainState {
    pub fn new() -> Self {
        let machines = vec![];

        MainState {
            machines,
            subdevices: vec![],
            hardware: HashMap::new(),
        }
    }

    pub fn generate_machine_hardware_from_serial(
        &mut self,
        path: &str,
    ) -> Result<(), anyhow::Error> {
        _ = path;
        todo!();
    }

    pub fn generate_machine_hardware_from_ethercat(
        &mut self,
        device_infos: &[MachineDeviceInfo],
        mapped_ecat_devices: Vec<(MetaSubdevice, Rc<RefCell<dyn EthercatDevice>>)>,
        ethercat_channel: EtherCATThreadChannel,
    ) {
        let device_map: HashMap<_, _> = mapped_ecat_devices
            .into_iter()
            .map(|(meta, device)| (meta.device_address, (meta, device)))
            .collect();

        let mut combined_list: Vec<_> = device_infos
            .iter()
            .map(|info| {
                let (meta, device) = device_map
                    .get(&info.device_address)
                    .expect("device should exist for every device_info");

                (*info, *meta, device.clone())
            })
            .collect();

        // sort the list
        combined_list.sort_by_key(|(device_info, _, _)| {
            (device_info.machine_id, device_info.machine_serial)
        });

        for (dev_info, _, ethercat_device) in combined_list.drain(..) {
            let identification = MachineIdentificationUnique::new(
                dev_info.machine_vendor,
                dev_info.machine_id,
                dev_info.machine_serial as u32,
            );

            let hardware = self
                .hardware
                .entry(identification)
                .or_insert_with(|| MachineHardware {
                    hw: Vec::new(),
                    ethercat_interface: Some(ethercat_channel.clone()),
                });

            hardware.hw.push(Hardware::Ethercat(IdentifiedEthercat {
                device: ethercat_device,
                ident: dev_info,
            }));
        }
    }
}
