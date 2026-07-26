use std::cell::RefCell;
use std::rc::Rc;

use qitech_lib::ethercat_hal::devices::EthercatDevice;
use qitech_lib::ethercat_hal::machine_ident_read::MachineDeviceInfo;
use qitech_lib::modbus::ModbusDevice;

#[derive(Clone)]
pub enum Hardware {
    Ethercat(EtherCATDeviceIdentified),
    ModbusRTU(ModbusRTUDeviceIdentified),
}

#[derive(Clone)]
pub struct EtherCATDeviceIdentified {
    pub device: Rc<RefCell<dyn EthercatDevice>>,
    pub info: MachineDeviceInfo,
}

#[derive(Clone)]
pub struct ModbusRTUDeviceIdentified {
    pub device: Rc<RefCell<dyn ModbusDevice>>,
    pub path: String,
}

// need a way to map device to machine identifcation unique
