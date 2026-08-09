use std::borrow::Cow;

use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

use crate::report::ConstraintViolationError;
use crate::report::ResourceKind;

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
    #[error("attempted to register resource not specified in the schema")]
    IllegalResourcePath { kind: ResourceKind, path: String },

    #[error("attempted to register resource with type other than specified in the schema")]
    IllegalResourceType {
        kind: ResourceKind,
        path: String,
        expected: String,
        received: String,
    },

    #[error("failed to configure hardware")]
    IllegalMachineType { expected: String, received: String },

    #[error("attempted to register resource {0} more than once")]
    DuplicateResource(String),

    #[error("resource expected {0} to be set")]
    MissingRequiredField(String),

    #[error("resource expected {0} to be set")]
    ConstraintViolation(#[from] ConstraintViolationError),
}

#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[error("{kind}")]
pub struct ActError {
    pub kind: ActErrorKind,
    pub impact: ActErrorImpact,
}

#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum ActErrorKind {
    #[error("hardware fault: {0}")]
    HardwareFault(String),
    #[error("validation failed: {0}")]
    ValidationFailed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActErrorImpact {
    /// Continue operating normally
    Ignore,

    /// Machine can operate, but with reduced capability
    Degraded,

    /// Machine cannot safely operate
    Irrecoverable,
}
