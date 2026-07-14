use std::fmt;
use qitech_lib::ethercat_hal::machine_ident_read::MachineDeviceInfo;
use serde::{Deserialize, Serialize};

use crate::vendors;

// --- unique ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MachineIdentificationUnique {
    pub vendor: u16,
    pub machine: u16,
    pub serial: u16,
}

impl MachineIdentificationUnique {
    pub const fn new(vendor: u16, machine: u16, serial: u16) -> Self {
        Self { vendor, machine, serial }
    }

    pub const fn to_u64(self) -> u64 {
        ((self.vendor as u64) << 48) | ((self.machine as u64) << 32) | (self.serial as u64)
    }

    pub const fn from_u64(value: u64) -> Self {
        Self {
            vendor: (value >> 48) as u16,
            machine: (value >> 32) as u16,
            serial: value as u16,
        }
    }

    pub const fn is_valid(self) -> bool {
        vendors::contains_id(self.vendor) && self.machine != 0 
    }
}

impl fmt::Display for MachineIdentificationUnique {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}/{}", self.vendor, self.machine, self.serial)
    }
}

// --- non-unique ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MachineIdentification {
    pub vendor: u16,
    pub machine: u16,
}

impl MachineIdentification {
    pub const fn new(vendor: u16, machine: u16) -> Self {
        Self { vendor, machine }
    }

    pub const fn to_u32(self) -> u32 {
        ((self.vendor as u32) << 16) | (self.machine as u32)
    }

    pub const fn from_u32(value: u32) -> Self {
        Self {
            vendor: (value >> 16) as u16,
            machine: value as u16,
        }
    }
}

impl From<MachineIdentificationUnique> for MachineIdentification {
    fn from(value: MachineIdentificationUnique) -> Self {
        Self { vendor: value.vendor, machine: value.machine }
    }
}

impl fmt::Display for MachineIdentification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.vendor, self.machine)
    }
}

// --- device --- 

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceIdentificationIdentified {
    pub device_machine_identification: DeviceMachineIdentification,
    pub device_hardware_identification: DeviceHardwareIdentification,
}

impl TryFrom<DeviceIdentification> for DeviceIdentificationIdentified {
    type Error = String;

    fn try_from(value: DeviceIdentification) -> Result<Self, Self::Error> {
        let device_machine_identification =
            value.device_machine_identification.ok_or("No device machine identification".to_string())?;

        Ok(Self {
            device_machine_identification,
            device_hardware_identification: value.device_hardware_identification,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceIdentification {
    pub device_machine_identification: Option<DeviceMachineIdentification>,
    pub device_hardware_identification: DeviceHardwareIdentification,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceMachineIdentification {
    pub machine_ident: MachineIdentificationUnique,
    pub role: u16,
}

impl From<MachineDeviceInfo> for DeviceMachineIdentification {
    fn from(value: MachineDeviceInfo) -> Self {
        DeviceMachineIdentification {
            machine_ident: MachineIdentificationUnique {
                vendor: value.machine_vendor,
                machine: value.machine_id,
                serial: value.machine_serial,
            },
            role: value.role,
        }
    }
}

impl DeviceMachineIdentification {
    /// Check if values are non-zero
    pub const fn is_valid(&self) -> bool {
        self.machine_ident.is_valid() && self.machine_ident.serial != 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceHardwareIdentification {
    Ethercat { subdevice_index: usize },
    Serial { path: String },
}
