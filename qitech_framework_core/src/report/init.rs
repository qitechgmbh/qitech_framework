use core::fmt;

use serde::Deserialize;
use serde::Serialize;

use crate::ident::DeviceIdentification;
use crate::ident::MachineIdentificationUnique;
use crate::report::error::BuildError;

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

    // --- xtrem ---
    XtremDiscoveryStarted,
    XtremBusFailed {
        error: String,
    },
    XtremDiscoveryCompleted {
        modules: Vec<XtremModuleMetadata>,
    },
    XtremDeviceNotFound {
        serial: u32,
    },
    /// Another module on the bus answers to the same `ID_O`. The bus routes replies by that
    /// field, so attaching both would cross-feed their readings.
    XtremDeviceIdCollision {
        serial: u32,
        device_id: u8,
    },
    XtremCouldNotInitialize {
        serial: u32,
        error: String,
    },

    // --- machine ---
    BuildingMachines,
    MachineBuildStarted {
        ident: MachineIdentificationUnique,
    },
    MachineBuildCompleted {
        ident: MachineIdentificationUnique,
        result: Result<(), BuildError>,
    },

    // --- finalizing ---
    EtherCATFinalizing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeInitStatus {
    NotStarted,
    EtherCATDiscovery,
    EtherCATInitializingDevices,
    ModbusRTUDiscovery,
    XtremDiscovery,
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
            RuntimeInitStatus::XtremDiscovery => "xtrem_discovery",
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
        use RuntimeInitEvent::*;

        match event {
            EtherCATDiscoveryStarted | EtherCATDiscoveryCompleted { .. } => {
                RuntimeInitStatus::EtherCATDiscovery
            }

            EtherCATStateUpdate(_)
            | EtherCATInitializationStarted
            | EtherCATDeviceInitializationFailed { .. }
            | EtherCATDeviceInitializationCompleted { .. } => {
                RuntimeInitStatus::EtherCATInitializingDevices
            }

            // --- modbus rtu ---
            ModbusRTUDiscoveryStarted
            | ModbusRTUDeviceNotFound { .. }
            | ModbusRTUCouldNotInitialize { .. } => RuntimeInitStatus::ModbusRTUDiscovery,

            // --- xtrem ---
            XtremDiscoveryStarted
            | XtremBusFailed { .. }
            | XtremDiscoveryCompleted { .. }
            | XtremDeviceNotFound { .. }
            | XtremDeviceIdCollision { .. }
            | XtremCouldNotInitialize { .. } => RuntimeInitStatus::XtremDiscovery,

            // --- building machines ---
            BuildingMachines | MachineBuildStarted { .. } | MachineBuildCompleted { .. } => {
                RuntimeInitStatus::BuildingMachines
            }

            // --- finishing ---
            EtherCATFinalizing => RuntimeInitStatus::Finalizing,
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

/// One XTREM module the discovery sweep answered from, claimed or not. Unclaimed modules are
/// reported too — that is how an installer finds the serial to configure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XtremModuleMetadata {
    /// Register `0000h`. Factory-set and unique, so this is the stable identity to configure on.
    pub serial: u32,
    pub device_id: u8,
    /// Rendered rather than a `SocketAddrV4`, so the report stays cheap to encode and display.
    pub addr: String,
    pub id_collision: bool,
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
