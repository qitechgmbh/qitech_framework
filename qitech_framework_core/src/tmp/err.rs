#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum MachineBuildError {
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

    #[error("expected an XTREM device at index {index}")]
    ExpectedXtremDeviceAtIndex { index: usize },

    #[error("failed to configure hardware: {0}")]
    HardwareConfig(String),

    #[error("device type mismatch at index {index}: expected {expected}")]
    DeviceTypeMismatch { index: usize, expected: String },

    // --- resource errors ---
    #[error("resource is not defined in the schema: {kind} at {path}")]
    IllegalResourcePath {
        kind: MachineResourceKind,
        path: String,
    },

    #[error(
        "resource type mismatch for {kind} at {path}: expected {expected}, received {received}"
    )]
    IllegalResourceType {
        kind: MachineResourceKind,
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