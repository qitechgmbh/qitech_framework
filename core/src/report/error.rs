use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum BuildError {
    #[error("machine required a valid ethercat interface")]
    MachineTypeNotRegistered,

    // --- hardware errors ---
    #[error("machine required a valid ethercat interface")]
    ExpectedEtherCATInterface,

    #[error("expected hardware at index {index}")]
    ExpectedHardwareAtIndex { index: usize },

    #[error("expected an ethercat device with role {role}")]
    ExpectedEtherCATDeviceWithRole { role: u16 },

    #[error("expected an ethercat device at index {index}")]
    ExpectedEtherCATDeviceAtIndex { index: usize },

    #[error("expected a serial device at index {index}")]
    ExpectedSerialDeviceAtIndex { index: usize },

    #[error("failed to configure hardware {0}")]
    HardwareConfig(String),

    #[error("device type mismatch at index {index}. Expected: {expected}")]
    DeviceTypeMismatch { index: usize, expected: String },

    // --- resource errors ---
    #[error("attempted to register resource {0} more than once")]
    DuplicateResource(String),

    #[error("resource expected {0} to be set")]
    MissingRequiredField(String),

    #[error("failed to configure hardware")]
    MachineTypeMismatch { expected: String, received: String },
}
