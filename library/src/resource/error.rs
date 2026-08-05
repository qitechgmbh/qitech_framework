use qitech_framework_core::ident::MachineIdentificationUnique;
use thiserror::Error;

use super::ResourceKind;

pub type RegisterResult<T> = Result<T, RegisterError>;

#[derive(Error, Debug)]
pub enum RegisterError {
    #[error("Duplicate resource")]
    Duplicate,

    #[error("Registry is full")]
    OutOfSlots,
}

#[derive(Error, Debug)]
pub enum ResourceAccessError {
    #[error("resource is not of the requested machine type")]
    MachineTypeMismatch,

    #[error("resource not found")]
    NoSuchResource,

    #[error("machine not found")]
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
