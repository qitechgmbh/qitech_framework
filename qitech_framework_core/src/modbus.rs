use serde::Deserialize;
use serde::Serialize;

use crate::ident::MachineInstanceIdentification;

/// A user-configured binding of a USB serial port to a machine instance, persisted to disk so it
/// survives restarts. Keyed by `port` (the `/dev/serial/by-path` basename), which stays stable
/// across replug unlike `/dev/ttyUSBn`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModbusRtuAssignment {
    pub port: String,
    pub machine: MachineInstanceIdentification,
    pub slave_id: u8,
}

/// One row of the Modbus RTU device table: what is plugged in (if anything), joined with what is
/// assigned (if anything). A port can be present without an assignment (freshly plugged in) or
/// assigned without being present (unplugged, moved to another port).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModbusRTUDeviceMetadata {
    pub port: String,
    pub present: bool,
    pub device_node: Option<String>,
    pub by_id: Option<String>,
    pub description: Option<String>,
    pub usb_vid: Option<u16>,
    pub usb_pid: Option<u16>,
    pub usb_serial: Option<String>,
    pub assignment: Option<ModbusRtuAssignment>,
}
