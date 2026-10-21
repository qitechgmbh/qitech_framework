use std::collections::BTreeMap;
use std::time::Duration;

use kdl::KdlDocument;
use serde::Deserialize;
use serde::Serialize;

use crate::schema::Issues;
use crate::system::SystemError;
use crate::system::parser;

// --- system schema ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemSchema {
    /// Imported machine types, e.g. `winder_v1`.
    pub imports: Vec<String>,
    /// Machine instance name -> imported machine type.
    pub machines: BTreeMap<String, String>,
    pub modbus_rtu: Vec<ModbusRtuBus>,
}

impl SystemSchema {
    pub fn parse_str(input: &str) -> Result<Self, SystemError> {
        let document: KdlDocument = input
            .parse()
            .map_err(|e: kdl::KdlError| SystemError::from_kdl(input, e))?;

        let mut issues = Issues::new(input);
        let system = parser::parse_system(&document, &mut issues);
        issues.finish(system)
    }
}

// --- modbus rtu ---
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModbusRtuBus {
    /// Serial port, e.g. `/dev/ttyUSB0`.
    pub path: String,
    pub baudrate: u32,
    pub parity: Option<Parity>,
    /// 1 or 2.
    pub stop_bits: Option<u8>,
    /// 5 to 8.
    pub data_bits: Option<u8>,
    /// Slave address (1..=247) -> device.
    pub devices: BTreeMap<u8, ModbusRtuDevice>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Parity {
    None,
    Even,
    Odd,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModbusRtuDevice {
    pub binding: Binding,
    pub timeout: Option<Duration>,
}

// --- binding ---
/// `just_some_laser.device`: a machine instance and a dotted path inside it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Binding {
    pub machine: String,
    pub path: String,
}
