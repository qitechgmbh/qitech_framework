use std::fmt;

use qitech_framework_common::MachineIdentificationUnique;

use super::ResourceKind;

pub type RegisterResult<T> = Result<T, RegisterError>;
pub type RegisterError = ResourceError<RegisterErrorKind>;

#[derive(Debug)]
pub enum RegisterErrorKind {
    MissingRequiredField(&'static str),
    Duplicate,
    RegistryFull,
    NameTooLarge,
}

pub type ResolveResult<T> = Result<T, ResolveError>;
pub type ResolveError = ResourceError<ResolveErrorKind>;

#[derive(Debug, Clone, Copy)]
pub enum ResolveErrorKind {
    NoSuchProperty,
    InvalidType,
}

#[derive(Debug)]
pub struct ResourceError<E> {
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

impl fmt::Display for ResolveErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolveErrorKind::NoSuchProperty => write!(f, "no such property"),
            ResolveErrorKind::InvalidType => write!(f, "invalid type"),
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
