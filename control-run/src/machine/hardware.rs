use std::{cell::RefCell, rc::Rc};
use qitech_lib::{ethercat_hal::{EtherCATThreadChannel, devices::EthercatDevice, machine_ident_read::MachineDeviceInfo}, modbus::ModbusDevice};

#[derive(Clone, Default)]
pub struct MachineHardware {
    pub hw: Vec<Hardware>,
    pub ethercat_interface: Option<EtherCATThreadChannel>,
}

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
    pub hw: Rc<RefCell<dyn ModbusDevice>>,
}
