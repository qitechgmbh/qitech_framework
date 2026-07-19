use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use qitech_lib::ethercat_hal::MetaSubdevice;
use qitech_lib::ethercat_hal::machine_ident_read::MachineDeviceInfo;
use qitech_lib::ethercat_hal::devices::EthercatDevice;
use qitech_lib::modbus::ModbusDevice;
use control_core::MachineIdentificationUnique;

pub type MachineHardwareRegistry = HashMap<MachineIdentificationUnique, Vec<Hardware>>;

#[derive(Clone)]
pub enum Hardware {
    Ethercat(IdentifiedEthercat),
    Modbus(IdentifiedModbus),
}

#[derive(Clone)]
pub struct IdentifiedEthercat {
    pub device: Rc<RefCell<dyn EthercatDevice>>,
    pub ident: MachineDeviceInfo,
}

#[derive(Clone)]
pub struct IdentifiedModbus {
    pub device: Rc<RefCell<dyn ModbusDevice>>,
}

pub fn append_ethercat(
    registry: &mut MachineHardwareRegistry,
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
            vendor: ident.machine_vendor,
            machine: ident.machine_id,
            serial: ident.machine_serial,
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
