use core::fmt;

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
    ModbusRTUDeviceNotFound {
        path: String,
    },

    ModbusRTUCouldNotInitialize {
        error: String,
    },

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeInitStatus {
    NotStarted,
    EtherCATDiscovery,
    EtherCATInitializingDevices,
    ModbusRTUDiscovery,
    BuildingMachines,
    Finalizing,
    Completed,
    Failed,
}

impl RuntimeInitStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            RuntimeInitStatus::NotStarted => "not_started",
            RuntimeInitStatus::EtherCATDiscovery => "ethercat_discovery",
            RuntimeInitStatus::EtherCATInitializingDevices => "ethercat_initializing_devices",
            RuntimeInitStatus::ModbusRTUDiscovery => "modbus_rtu_discovery",
            RuntimeInitStatus::BuildingMachines => "building_machines",
            RuntimeInitStatus::Finalizing => "finalizing",
            RuntimeInitStatus::Completed => "completed",
            RuntimeInitStatus::Failed => "failed",
        }
    }
}

impl fmt::Display for RuntimeInitStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&RuntimeInitEvent> for RuntimeInitStatus {
    fn from(event: &RuntimeInitEvent) -> Self {
        match event {
            RuntimeInitEvent::EtherCATDiscoveryStarted => RuntimeInitStatus::EtherCATDiscovery,

            RuntimeInitEvent::EtherCATDiscoveryCompleted { .. } => {
                RuntimeInitStatus::EtherCATInitializingDevices
            }

            RuntimeInitEvent::EtherCATStateUpdate(_) => {
                RuntimeInitStatus::EtherCATInitializingDevices
            }

            RuntimeInitEvent::EtherCATInitializationStarted => {
                RuntimeInitStatus::EtherCATInitializingDevices
            }

            RuntimeInitEvent::EtherCATDeviceInitializationFailed { .. } => {
                RuntimeInitStatus::Failed
            }

            RuntimeInitEvent::EtherCATDeviceInitializationCompleted { .. } => {
                RuntimeInitStatus::ModbusRTUDiscovery
            }

            RuntimeInitEvent::ModbusRTUDiscoveryStarted => RuntimeInitStatus::ModbusRTUDiscovery,

            RuntimeInitEvent::ModbusRTUDeviceNotFound { .. } => RuntimeInitStatus::ModbusRTUDiscovery,

            RuntimeInitEvent::BuildingMachines => RuntimeInitStatus::BuildingMachines,

            RuntimeInitEvent::BuiltMachine { .. } => RuntimeInitStatus::BuildingMachines,

            RuntimeInitEvent::FailedToBuildMachine { .. } => RuntimeInitStatus::Failed,

            RuntimeInitEvent::EtherCATFinalizing => RuntimeInitStatus::Finalizing,

            RuntimeInitEvent::Finished => RuntimeInitStatus::Completed,
        }
    }
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
