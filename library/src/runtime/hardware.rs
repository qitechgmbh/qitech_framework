use std::{cell::RefCell, collections::HashMap, rc::Rc};
use qitech_framework_common::{MachineIdentification, MachineIdentificationUnique};
use qitech_lib::ethercat_hal::{MetaSubdevice, devices::EthercatDevice, machine_ident_read::MachineDeviceInfo};

pub struct MachineHardwareRegistry {
    inner: HashMap<MachineIdentificationUnique, Vec<Hardware>>,
}

impl MachineHardwareRegistry {
    pub fn append_ethercat(
        &mut self,
        device_infos: &[MachineDeviceInfo],
        mapped_ecat_devices: &Vec<(MetaSubdevice, Rc<RefCell<dyn EthercatDevice + 'static>>)>,
    ) {
        let combined_list = create_mapped_ethercat_devices(
            device_infos, 
            mapped_ecat_devices
        );

        for mapped in combined_list {
            let ident = mapped.info;

            let identification = MachineIdentificationUnique {
                serial: ident.machine_serial,
                identification: MachineIdentification {
                    vendor_id: ident.machine_vendor,
                    machine_id: ident.machine_id,
                },
            };

            registry
                .entry(identification)
                .or_default()
                .push(Hardware::Ethercat(IdentifiedEthercat {
                    device: mapped.device,
                    ident,
                }));
        }
    }
}

struct MappedEthercatDevice {
    info: MachineDeviceInfo,
    device: Rc<RefCell<dyn EthercatDevice>>,
}

fn create_mapped_ethercat_devices(
    device_infos: &[MachineDeviceInfo],
    mapped_ecat_devices: &[(MetaSubdevice, Rc<RefCell<dyn EthercatDevice>>)],
) -> Vec<MappedEthercatDevice> {
    let mut result = Vec::new();

    for info in device_infos {
        for (meta, device) in mapped_ecat_devices {
            if meta.device_address == info.device_address {
                result.push(MappedEthercatDevice {
                    info: *info,
                    device: device.clone(),
                });
                break;
            }
        }
    }

    result.sort_by_key(|device| {
        (device.info.machine_id, device.info.machine_serial)
    });

    result
}
