use qitech_framework_common::MachineIdentificationUnique;
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

#[derive(Debug, Error)]
#[error("{resource_kind} '{resource_path}' on machine {machine_ident}: {error}")]
pub struct ResourceError<E>
where
    E: std::error::Error + 'static,
{
    pub machine_ident: MachineIdentificationUnique,
    pub resource_kind: ResourceKind,
    pub resource_path: &'static str,

    #[source]
    pub error: E,
}
