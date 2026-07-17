use std::str::FromStr;
use serde::de::Error;

pub use version::QmsVersion;

// expose latest version directly
mod version;
mod migration;

pub mod v1_0;
pub use v1_0 as latest;

pub type ParseError = yaml_serde::Error;

pub fn parse_latest(data: &str) -> Result<latest::MachineSchema, ParseError> {
    match AnyMachineSchema::from_str(data)? {
        AnyMachineSchema::UndefinedVersion(qms_version) => Err(ParseError::custom(format!(
            "Undefined version: {qms_version}"
        ))),
        AnyMachineSchema::UnsupportedVersion(qms_version) => Err(ParseError::custom(format!(
            "Unsupported version: {qms_version}"
        ))),
        // if new versions are added use migration strategies here to try
        // and parse into the latest schema version, if not possible/available fail.
        AnyMachineSchema::V1_0(schema) => Ok(schema),
    }
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum AnyMachineSchema {
    UndefinedVersion(QmsVersion),
    UnsupportedVersion(QmsVersion),
    V1_0(v1_0::MachineSchema),
}

impl FromStr for AnyMachineSchema {
    type Err = ParseError;

    fn from_str(data: &str) -> Result<Self, Self::Err> {
        let first_line = data
            .lines()
            .find(|line| !line.trim().is_empty())
            .ok_or(yaml_serde::Error::custom("empty input"))?;

        let value = first_line
            .strip_prefix("qms_version:")
            .ok_or(yaml_serde::Error::custom("first line must be `qms_version: <version>`"))?
            .trim()
            .trim_matches('"')
            .trim_matches('\'');

        let version = yaml_serde::from_str::<QmsVersion>(value)?;

        if version.is_unsupported() {
            return Ok(AnyMachineSchema::UnsupportedVersion(version));
        }

        if !version.is_supported() && !version.is_deprecated() {
            return Ok(AnyMachineSchema::UndefinedVersion(version));
        }

        match version {
            v1_0::VERSION => {
                v1_0::parse(data)
            }, 
            _ => unreachable!(""),
        }
    }
}
