use qitech_framework_core::ident::MachineIdentificationUnique;
use thiserror::Error;

use super::ResourceKind;

pub type RegisterResult<T> = Result<T, RegisterError>;

#[derive(Error, Debug)]
pub enum RegisterError {
    #[error("Missing required field {0}")]
    MissingRequiredField(&'static str),

    #[error("Duplicate resource")]
    Duplicate,

    #[error("Registry is full")]
    RegistryFull,
}

#[derive(Error, Debug)]
pub enum ResourceAccessError {
    #[error("resource not found")]
    MachineTypeMismatch,

    #[error("resource not found")]
    NoSuchResource,

    #[error("resource not found")]
    NoSuchMachine,
}

#[derive(Debug, Error)]
#[error("{resource_kind} '{resource_name}' on machine {machine_ident}: {error}")]
pub struct ResourceError<E>
where
    E: std::error::Error + 'static,
{
    pub machine_ident: MachineIdentificationUnique,
    pub resource_kind: ResourceKind,
    pub resource_name: &'static str,

    #[source]
    pub error: E,
}
