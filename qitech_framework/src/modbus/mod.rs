use std::collections::HashMap;
use std::path::PathBuf;

use serialport::DataBits;
use serialport::Parity;
use serialport::StopBits;

// Goals:
// -> user doesn't invoke

// --- config ---
pub struct ModbusRTUBusConfig {
    pub(crate) port: ModbusRtuPort,
    pub(crate) baud_rate: u32,
    pub(crate) data_bits: DataBits,
    pub(crate) stop_bits: StopBits,
    pub(crate) parity: Parity,
    pub(crate) devices: HashMap<u8, ModbusRTUDeviceConfig>,
}

impl ModbusRTUBusConfig {
    pub fn new(port: ModbusRtuPort, baud_rate: u32) -> Self {
        Self {
            port,
            baud_rate,
            data_bits: DataBits::Eight,
            stop_bits: StopBits::One,
            parity: Parity::None,
            devices: Default::default(),
        }
    }

    pub fn data_bits(mut self, data_bits: DataBits) -> Self {
        self.data_bits = data_bits;
        self
    }

    pub fn stop_bits(mut self, stop_bits: StopBits) -> Self {
        self.stop_bits = stop_bits;
        self
    }

    pub fn parity(mut self, parity: Parity) -> Self {
        self.parity = parity;
        self
    }

    pub fn device(mut self, slave_id: u8, config: ModbusRTUDeviceConfig) -> Self {
        // TODO: enable feature by fixing in qitech_lib
        assert!(self.devices.is_empty(), "Currently unsupported");
        self.devices.insert(slave_id, config);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModbusRtuPort {
    /// Direct device path, e.g. `/dev/ttyUSB0`.
    Device(PathBuf),

    /// Stable USB topology path, e.g.
    /// `pci-0000:c6:00.0-usb-0:2.1:1.0-port0`.
    UsbTopology(String),
}

#[derive(Debug, Clone, Copy)]
pub struct ModbusRTUDeviceConfig {
    /// Machine instance this Modbus device is assigned to.
    pub instance: u32,

    /// Identifier used by the machine to distinguish this device from
    /// other devices of the same type assigned to the same machine.
    pub ident: u16,
}

/// Manager for everything modbus related.
/// Also responsible for hardware scanning
pub struct ModbusManager {
    pub(crate) rtu_buses: HashMap<u8, ModbusRTUDeviceConfig>,
}

pub struct ModbusRtuBusManager {
    pub(crate) devices: HashMap<u8, ModbusRTUDeviceConfig>,
}
