use std::{cell::RefCell, rc::Rc};
use qitech_lib::{ethercat_hal::{devices::EthercatDevice, machine_ident_read::MachineDeviceInfo}, modbus::ModbusDevice};
use control_core::MachineIdentificationUnique;

use crate::resource::{
    ConfigPropertyReader, ConfigPropertyResolver, MeasurementReader, MeasurementResolver,
    ResolveError, StatePropertyReader, StatePropertyResolver,
};

// --- act ---
#[derive(Debug)]
pub struct ActError {
    pub kind: ActErrorKind,
    pub recoverable: bool,
    pub explanation: String,
}

#[derive(Debug)]
pub enum ActErrorKind {
    HardwareFault,
    InvariantViolation,
}

// --- react ---
pub struct ReactContext<'a> {
    pub config: ConfigPropertyReader<'a>,
    pub state: StatePropertyReader<'a>,
    pub measurements: MeasurementReader<'a>,
}

#[derive(Debug)]
pub struct ReactError {
    pub kind: ReactErrorKind,
    pub recoverable: bool,
    pub explanation: String,
}

#[derive(Debug)]
pub enum ReactErrorKind {
    HardwareFault,
    InvariantViolation,
    ExpiredHandle,
}

// --- subscribe ---
pub struct SubscribeContext<'a> {
    pub ident: MachineIdentificationUnique,
    pub config: ConfigPropertyResolver<'a>,
    pub state: StatePropertyResolver<'a>,
    pub measurements: MeasurementResolver<'a>,
}

#[derive(Debug)]
pub enum SubscribeError {
    OperationNotSupported,
    UnsupportedMachine,
    TooManySubscriptions,
    NoSuchResource,
    InvalidResourceType,
}

impl From<ResolveError> for SubscribeError {
    fn from(value: ResolveError) -> Self {
        match value {
            ResolveError::NoSuchProperty => SubscribeError::NoSuchResource,
            ResolveError::InvalidType => SubscribeError::InvalidResourceType,
        }
    }
}

// --- hardware ---
#[derive(Clone)]
pub enum Hardware {
    Ethercat(IdentifiedEthercat),
    Modbus(IdentifiedModbus),
}

#[derive(Clone)]
pub struct IdentifiedEthercat {
    pub device: Rc<RefCell<dyn EthercatDevice>>,
    pub ident: MachineDeviceInfo,
}

#[derive(Clone)]
pub struct IdentifiedModbus {
    pub device: Rc<RefCell<dyn ModbusDevice>>,
}
