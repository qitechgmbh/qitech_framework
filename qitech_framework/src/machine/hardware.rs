use std::cell::RefCell;
use std::rc::Rc;

use qitech_lib::ethercat_hal::devices::EthercatDevice;
use qitech_lib::ethercat_hal::machine_ident_read::MachineDeviceInfo;
use qitech_lib::modbus::ModbusDevice;
use qitech_lib::xtrem::XtremDevice;
use qitech_lib::xtrem::XtremProbe;

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
}

#[derive(Clone)]
pub struct XtremDeviceIdentified {
    pub device: Rc<RefCell<dyn XtremDevice>>,

    /// What the discovery sweep learned: serial, device id, and unicast address. Kept so a
    /// machine can surface its module's identity without probing the bus again.
    pub probe: XtremProbe,
}
