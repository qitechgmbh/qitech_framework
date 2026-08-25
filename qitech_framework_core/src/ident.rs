use std::fmt;

use serde::Deserialize;
use serde::Serialize;

use crate::vendors;

// --- instance ident ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MachineInstanceIdentification {
    pub machine: MachineIdentification,
    pub serial: u16,
}

impl MachineInstanceIdentification {
    pub const fn from_ident(machine: MachineIdentification, serial: u16) -> Self {
        Self { machine, serial }
    }

    pub const fn to_u64(self) -> u64 {
        ((self.machine.vendor_id as u64) << 48)
            | ((self.machine.machine_id as u64) << 32)
            | (self.serial as u64)
    }

    pub const fn from_u64(value: u64) -> Self {
        Self {
            machine: MachineIdentification {
                vendor_id: (value >> 48) as u16,
                machine_id: (value >> 32) as u16,
            },
            serial: value as u16,
        }
    }

    pub const fn is_valid(self) -> bool {
        self.machine.is_valid() && self.machine.machine_id != 0
    }
}

impl fmt::Display for MachineInstanceIdentification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // let vendor_name = match vendors::get_name(self.identification.vendor_id) {
        //     Some(v) => v,
        //     None => &self.identification.vendor_id.to_string(),
        // };

        write!(
            f,
            "{}:{}:{}",
            self.machine.vendor_id, self.machine.machine_id, self.serial
        )
    }
}

// --- ident ---
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

    pub const fn unique(self, serial: u16) -> MachineInstanceIdentification {
        MachineInstanceIdentification {
            machine: self,
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
pub struct DeviceIdentification {
    pub assignment: Option<DeviceMachineAssignment>,
    pub hardware: DeviceHardwareIdentification,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceMachineAssignment {
    pub machine: MachineInstanceIdentification,
    pub role: u16,
}

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
