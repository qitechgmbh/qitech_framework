use std::fmt;

use serde::Deserialize;
use serde::Serialize;

use crate::vendors;

// --- with instance/serial id ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MachineIdentificationUnique {
    pub identification: MachineIdentification,
    pub serial: u16,
}

impl MachineIdentificationUnique {
    pub const fn from_ident(identification: MachineIdentification, serial: u16) -> Self {
        Self {
            identification,
            serial,
        }
    }

    pub const fn to_u64(self) -> u64 {
        ((self.identification.vendor_id as u64) << 48)
            | ((self.identification.machine_id as u64) << 32)
            | (self.serial as u64)
    }

    pub const fn from_u64(value: u64) -> Self {
        Self {
            identification: MachineIdentification {
                vendor_id: (value >> 48) as u16,
                machine_id: (value >> 32) as u16,
            },
            serial: value as u16,
        }
    }

    pub const fn is_valid(self) -> bool {
        self.identification.is_valid() && self.identification.machine_id != 0
    }
}

impl fmt::Display for MachineIdentificationUnique {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // let vendor_name = match vendors::get_name(self.identification.vendor_id) {
        //     Some(v) => v,
        //     None => &self.identification.vendor_id.to_string(),
        // };

        write!(
            f,
            "{}:{}:{}",
            self.identification.vendor_id, self.identification.machine_id, self.serial
        )
    }
}

// --- regular ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MachineIdentification {
    pub vendor_id: u16,
    pub machine_id: u16,
}

impl MachineIdentification {
    pub const fn new(vendor_id: u16, machine_id: u16) -> Self {
        Self {
            vendor_id,
            machine_id,
        }
    }

    pub const fn is_valid(self) -> bool {
        vendors::contains_id(self.vendor_id)
    }

    pub const fn unique(self, serial: u16) -> MachineIdentificationUnique {
        MachineIdentificationUnique {
            identification: self,
            serial,
        }
    }
}

impl fmt::Display for MachineIdentification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let vendor_name = match vendors::get_name(self.vendor_id) {
            Some(v) => v,
            None => &self.vendor_id.to_string(),
        };

        write!(f, "{vendor_name}:{}", self.machine_id)
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
        let device_machine_identification = value
            .device_machine_identification
            .ok_or("No device machine identification".to_string())?;

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

impl DeviceMachineIdentification {
    /// Check if values are non-zero
    pub const fn is_valid(&self) -> bool {
        self.machine_ident.is_valid() && self.machine_ident.serial != 0
    }
}

// impl From<MachineDeviceInfo> for DeviceMachineIdentification {
//     fn from(value: MachineDeviceInfo) -> Self {
//         DeviceMachineIdentification {
//             machine_identification_unique: QiTechMachineIdentificationUnique {
//                 machine_identification: MachineIdentification {
//                     vendor: value.machine_vendor,
//                     machine: value.machine_id,
//                 },
//                 serial: value.machine_serial,
//             },
//             role: value.role,
//         }
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceHardwareIdentification {
    Ethercat(DeviceHardwareIdentificationEthercat),
    Serial(DeviceHardwareIdentificationSerial),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceHardwareIdentificationEthercat {
    pub subdevice_index: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceHardwareIdentificationSerial {
    pub path: String,
}
