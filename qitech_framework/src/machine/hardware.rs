use std::cell::RefCell;
use std::rc::Rc;
use qitech_lib::ethercat_hal_devices::EthercatDevice;
use qitech_lib::xtrem::XtremDevice;
use qitech_lib::xtrem::XtremProbe;
use crate::machine::modbus::ModbusDevice;

#[derive(Debug, Copy, Clone)]
pub struct MachineDeviceInfo {
    pub role: u16,
    pub machine_id: u16,
    pub machine_vendor: u16,
    pub machine_serial: u16,
    pub device_address: u16,
}

#[derive(Clone)]
pub enum Hardware {
    Ethercat(EtherCATDeviceIdentified),
    ModbusRTU(ModbusRTUDeviceIdentified),
    Xtrem(XtremDeviceIdentified),
}

#[derive(Clone)]
pub struct EtherCATDeviceIdentified {
    pub device: Rc<RefCell<dyn EthercatDevice>>,
    pub info: MachineDeviceInfo,
}

#[derive(Clone)]
pub struct ModbusRTUDeviceIdentified {
    pub device: Rc<RefCell<dyn ModbusDevice>>,

    /// topology path under `/dev/serial/by-path` the device is bound to
    pub binding: String,
}

#[derive(Clone)]
pub struct XtremDeviceIdentified {
    pub device: Rc<RefCell<dyn XtremDevice>>,
    pub probe: XtremProbe,
}
