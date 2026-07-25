use std::cell::RefCell;
use std::rc::Rc;

use qitech_lib::ethercat_hal::devices::EthercatDevice;
use qitech_lib::ethercat_hal::machine_ident_read::MachineDeviceInfo;
use qitech_lib::modbus::ModbusDevice;

#[derive(Clone)]
pub enum Hardware {
    Ethercat(EtherCATDeviceIdentified),
    Modbus(ModbusDeviceIdentified),
}

#[derive(Clone)]
pub struct EtherCATDeviceIdentified {
    pub device: Rc<RefCell<dyn EthercatDevice>>,
    pub ident: MachineDeviceInfo,
}

#[derive(Clone)]
pub struct ModbusDeviceIdentified {
    pub device: Rc<RefCell<dyn ModbusDevice>>,
}
