use std::fmt;
use super::Kind;

pub type RegisterResult<T> = Result<T, RegisterError>;
pub type RegisterError = Error<RegisterErrorKind>;

#[derive(Debug)]
pub enum RegisterErrorKind {
    AlreadyRegistered,
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