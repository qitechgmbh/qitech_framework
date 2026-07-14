use qitech_lib::ethercat_hal;
use serde::{Deserialize, Serialize};

use crate::ident::DeviceIdentification;

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum EtherCATState {
    NoInterface,
    Boot,
    Init,
    PreOp,
    PreopPdi,
    Op,
}

impl From<ethercat_hal::EtherCATState> for EtherCATState {
    fn from(val: ethercat_hal::EtherCATState) -> Self {
        use ethercat_hal::EtherCATState::*;
        match val {
            NoInterface => EtherCATState::NoInterface,
            Boot => EtherCATState::Boot,
            Init => EtherCATState::Init,
            PreOp => EtherCATState::PreOp,
            PreopPdi => EtherCATState::PreopPdi,
            Op => EtherCATState::Op,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum RuntimeStateEvent {
    Initializing,
    EtherCATDiscoveryStarted,
    EtherCATDiscoveryCompleted { interface_name: String },
    EtherCATDevicesInitializing,
    EtherCATDevicesStateChanged { new_state: EtherCATState },
    EtherCATDevicesCompleted { devices: Vec<EtherCatDeviceMetaData> },
    Running,
    Exiting,
}

// #[derive(Clone)]
// pub enum RuntimeEvent {
//     Started,
//     MachinesEvent(MachinesEvent),
//     EthercatDevicesEvent(EthercatDevicesEvent),
//     EthercatInterfaceDiscoveryEvent(EthercatInterfaceDiscoveryEvent),
// }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EthercatDevicesEvent {
    Initializing,
    Complete { devices: Vec<EtherCatDeviceMetaData> },
    Error(String),
    State(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EthercatInterfaceDiscoveryEvent {
    Discovering(bool),
    Done(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EtherCatDeviceMetaData {
    pub configured_address: u16,
    pub name: String,
    pub vendor_id: u32,
    pub product_id: u32,
    pub revision: u32,
    pub device_identification: DeviceIdentification,
}
