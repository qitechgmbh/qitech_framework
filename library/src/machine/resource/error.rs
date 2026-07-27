use std::fmt;

use qitech_framework_common::MachineIdentificationUnique;

use super::ResourceKind;

pub type RegisterResult<T> = Result<T, RegisterErrorKind>;
pub type RegisterError = ResourceError<RegisterErrorKind>;

#[derive(Debug)]
pub enum RegisterErrorKind {
    MissingRequiredField(&'static str),
    Duplicate,
    RegistryFull,
    NameTooLarge,
}

pub type SubscribeResult<T> = Result<T, SubscribeErrorKind>;
pub type SubscribeError = ResourceError<SubscribeErrorKind>;

#[derive(Debug, Clone, Copy)]
pub enum SubscribeErrorKind {
    Duplicate,
    InvalidType,
    NoSuchProperty,
}

#[derive(Debug)]
pub struct ResourceError<E> {
    pub machine_ident: MachineIdentificationUnique,
    pub resource_kind: ResourceKind,
    pub resource_path: &'static str,
    pub error_kind: E,
}

pub type ReadResult<T> = Result<T, ReadError>;

#[derive(Debug)]
pub struct ReadError;

#[derive(Debug, Clone, Copy)]
pub struct HandleError {
    pub resource_kind: ResourceKind,
    pub resource_path: &'static str,
    pub machine_ident: MachineIdentificationUnique,
}

// --- impl display ---
impl fmt::Display for RegisterErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegisterErrorKind::MissingRequiredField(field) => {
                write!(f, "missing required field `{field}`")
            }
            RegisterErrorKind::Duplicate => write!(f, "duplicate property"),
            RegisterErrorKind::RegistryFull => write!(f, "registry full"),
            RegisterErrorKind::NameTooLarge => write!(f, "name too large"),
        }
    }
}

impl fmt::Display for SubscribeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubscribeErrorKind::NoSuchProperty => write!(f, "no such property"),
            SubscribeErrorKind::InvalidType => write!(f, "invalid type"),
        }
    }
}

impl<K: fmt::Display> fmt::Display for ResourceError<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} error for {:?} at '{}'",
            self.error_kind, self.resource_kind, self.resource_path
        )
    }
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("expired handle")
    }
}

impl fmt::Display for HandleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} `{}` handle is no longer valid: it was unregistered",
            self.resource_kind, self.resource_path,
        )
    }
}

// --- impl error ---
impl<K: fmt::Display + fmt::Debug> std::error::Error for ResourceError<K> {}
impl std::error::Error for ReadError {}
impl std::error::Error for HandleError {}
