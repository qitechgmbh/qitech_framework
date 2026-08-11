use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

use crate::report::ConstraintViolationError;
use crate::report::ResourceKind;

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum BuildError {
    // --- machine / hardware errors ---
    #[error("machine type is not registered")]
    MachineTypeNotRegistered,

    #[error("expected a valid EtherCAT interface")]
    ExpectedEtherCATInterface,

    #[error("expected a valid EtherCAT interface: {0}")]
    EtherCATConfigureError(String),

    #[error("expected hardware at index {index}")]
    ExpectedHardwareAtIndex { index: usize },

    #[error("expected an EtherCAT device with role {role}")]
    ExpectedEtherCATDeviceWithRole { role: u16 },

    #[error("expected an EtherCAT device at index {index}")]
    ExpectedEtherCATDeviceAtIndex { index: usize },

    #[error("expected a serial device at index {index}")]
    ExpectedSerialDeviceAtIndex { index: usize },

    #[error("failed to configure hardware: {0}")]
    HardwareConfig(String),

    #[error("device type mismatch at index {index}: expected {expected}")]
    DeviceTypeMismatch { index: usize, expected: String },

    // --- resource errors ---
    #[error("resource is not defined in the schema: {kind} at {path}")]
    IllegalResourcePath { kind: ResourceKind, path: String },

    #[error(
        "resource type mismatch for {kind} at {path}: expected {expected}, received {received}"
    )]
    IllegalResourceType {
        kind: ResourceKind,
        path: String,
        expected: String,
        received: String,
    },

    #[error("machine type mismatch: expected {expected}, received {received}")]
    IllegalMachineType { expected: String, received: String },

    #[error("resource registered more than once: {0}")]
    DuplicateResource(String),

    #[error("required resource field is missing: {0}")]
    MissingRequiredField(String),

    #[error("constraint violation: {0}")]
    ConstraintViolation(#[from] ConstraintViolationError),
}

impl From<anyhow::Error> for BuildError {
    fn from(err: anyhow::Error) -> Self {
        BuildError::EtherCATConfigureError(err.to_string())
    }
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

    #[error(transparent)]
    ConstraintViolation(#[from] ConstraintViolationError),

    #[error("{0}")]
    Custom(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActErrorImpact {
    /// Continue operating normally.
    Ignore,

    /// Machine can operate, but with reduced capability.
    Degraded,

    /// Machine cannot safely operate.
    Irrecoverable,
}
