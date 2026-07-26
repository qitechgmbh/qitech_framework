use std::cell::RefCell;
use std::rc::Rc;

use crossbeam::channel::Sender;
use qitech_framework_common::MachineIdentification;
use qitech_framework_common::MachineIdentificationUnique;
use qitech_lib::modbus::ModbusDevice;
use qitech_lib::modbus::devices::qitech_laser::LaserDevice;
use serialport::available_ports;

use crate::machine::Hardware;
use crate::machine::hardware::ModbusRTUDeviceIdentified;
use crate::runtime::MachineRegistry;

pub fn run(
    machine_registry: MachineRegistry,
    tx: Sender<(MachineIdentificationUnique, Vec<Hardware>)>,
) {
    loop {
        let Ok(ports) = available_ports() else {
            continue;
        };

        for port in ports {
            if port.port_name != "/dev/ttyUSB0" && port.port_name != "/dev/ttyUSB1" {
                continue;
            }

            let device = LaserDevice::new(port.port_name.to_owned(), 1, None).unwrap();
            let device = Rc::new(RefCell::new(device));

            let id_modbus = ModbusRTUDeviceIdentified {
                device,
                path: todo!(),
            };

            // ident of laser_v1
            let ident = MachineIdentificationUnique {
                identification: MachineIdentification {
                    vendor_id: 1,
                    machine_id: 6,
                },
                serial: 1,
            };
        }
    }
}
