use std::cell::RefCell;
use std::rc::Rc;

use qitech_framework_core::ident::MachineIdentificationUnique;
use qitech_lib::ethercat_hal::MetaSubdevice;
use qitech_lib::ethercat_hal::devices::EthercatDevice;
use qitech_lib::modbus::ModbusDevice;

#[derive(Clone)]
pub enum Hardware {
    Ethercat(EtherCATDeviceIdentified),
    ModbusRTU(ModbusRTUDeviceIdentified),
}

#[derive(Clone)]
pub struct EtherCATDeviceIdentified {
    pub meta: MetaSubdevice,
    pub handle: Rc<RefCell<dyn EthercatDevice>>,
    pub ident: MachineIdentificationUnique,
    pub role: Option<u16>,
}

#[derive(Clone)]
pub struct ModbusRTUDeviceIdentified {
    pub device: Rc<RefCell<dyn ModbusDevice>>,
}
