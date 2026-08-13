use core::fmt;
use std::str::FromStr;

use serde::Deserialize;
use serde::Deserializer;
use serde::de;
use serde::de::Visitor;

#[derive(Debug)]
pub struct VersionRaw {
    pub major: u32,
    pub minor: u32,
}

impl FromStr for VersionRaw {
    type Err = &'static str;

    /// Parses a version string in strict `"major.minor"` format,
    /// e.g. `"1.0"`. Rejects missing components, non-numeric
    /// components, and any extra `.`-separated segments.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut parts = value.split('.');

        let major = parts
            .next()
            .ok_or("missing major version")?
            .parse::<u32>()
            .map_err(|_| "invalid major version")?;

        let minor = parts
            .next()
            .ok_or("missing minor version")?
            .parse::<u32>()
            .map_err(|_| "invalid minor version")?;

        // Reject trailing components like "1.0.0". Only major.minor is valid.
        if parts.next().is_some() {
            return Err("too many version components");
        }

        Ok(Self { major, minor })
    }
}

impl<'de> Deserialize<'de> for VersionRaw {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VersionVisitor;

        impl Visitor<'_> for VersionVisitor {
            type Value = VersionRaw;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a language version in the format major.minor")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                value.parse().map_err(E::custom)
            }
        }

        deserializer.deserialize_str(VersionVisitor)
    }
}
