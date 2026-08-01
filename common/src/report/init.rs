use serde::Deserialize;
use serde::Serialize;

use crate::ident::DeviceIdentification;
use crate::ident::MachineIdentificationUnique;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeInitEvent {
    // --- ether cat discovery ---
    EtherCATDiscoveryStarted,
    EtherCATDiscoveryCompleted {
        interface: String,
    },

    // --- ether cat misc ---
    EtherCATStateUpdate(EtherCATStatus),

    // --- ether cat device ---
    EtherCATInitializationStarted,
    EtherCATDeviceInitializationFailed {
        error: String,
    },
    EtherCATDeviceInitializationCompleted {
        devices: Vec<EtherCATDeviceMetadata>,
    },

    // --- modbus rtu ---
    ModbusRTUDiscoveryStarted,

    // --- machine ---
    BuildingMachines,
    BuiltMachine {
        ident: MachineIdentificationUnique,
    },
    FailedToBuildMachine {
        ident: MachineIdentificationUnique,
    },

    // --- finalizing ---
    EtherCATFinalizing,
    Finished,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EtherCATStatus {
    NoInterface,
    Boot,
    Init,
    PreOp,
    PreopPdi,
    Op,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtherCATDeviceMetadata {
    pub configured_address: u16,
    pub name: String,
    pub vendor_id: u32,
    pub product_id: u32,
    pub revision: u32,
    pub device_identification: DeviceIdentification,
}
