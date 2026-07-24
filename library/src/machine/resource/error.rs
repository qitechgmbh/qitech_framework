use std::fmt;

use qitech_framework_common::MachineIdentificationUnique;

use super::Kind;

pub type RegisterResult<T> = Result<T, RegisterError>;
pub type RegisterError = Error<RegisterErrorKind>;

#[derive(Debug)]
pub enum RegisterErrorKind {
    Duplicate,
    RegistryFull,
    NameTooLarge,
}

pub type ResolveResult<T> = Result<T, ResolveError>;
pub type ResolveError = Error<ResolveErrorKind>;

#[derive(Debug, Clone, Copy)]
pub enum ResolveErrorKind {
    NoSuchProperty,
    InvalidType,
}

#[derive(Debug)]
pub struct Error<K> {
    pub resource_kind: Kind,
    pub resource_path: &'static str,
    pub kind: K,
}

pub type ReadResult<T> = Result<T, ReadError>;

#[derive(Debug)]
pub struct ReadError;

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("expired handle")
    }
}

impl std::error::Error for ReadError {}

#[derive(Debug)]
pub struct HandleError {
    pub resource_kind: Kind,
    pub resource_path: &'static str,
    pub machine_ident: MachineIdentificationUnique,
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

impl std::error::Error for HandleError {}
