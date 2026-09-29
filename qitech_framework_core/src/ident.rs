use std::fmt;

use serde::Deserialize;
use serde::Serialize;

// --- machine type id ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MachineTypeId(u16);

impl MachineTypeId {
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

impl From<u16> for MachineTypeId {
    fn from(value: u16) -> Self {
        Self::new(value)
    }
}

impl From<MachineTypeId> for u16 {
    fn from(value: MachineTypeId) -> Self {
        value.get()
    }
}

impl fmt::Display for MachineTypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// --- machine instance id ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MachineInstanceId {
    pub machine_type: MachineTypeId,
    pub instance_id: u16,
}

impl From<MachineInstanceId> for u32 {
    fn from(value: MachineInstanceId) -> Self {
        ((u16::from(value.machine_type) as u32) << 16) | (value.instance_id as u32)
    }
}

impl From<u32> for MachineInstanceId {
    fn from(value: u32) -> Self {
        Self {
            machine_type: MachineTypeId::from((value >> 16) as u16),
            instance_id: value as u16,
        }
    }
}

impl fmt::Display for MachineInstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.machine_type, self.instance_id)
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
    pub machine: MachineInstanceId,
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
