use std::{fmt, str::FromStr};
use serde::{Deserialize, Deserializer, Serialize, de::{self, Visitor}};

/// QiTech Machine Schema Version used for tracking schema changes
/// and ensuring the correct format is used for deserializing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct QmsVersion {
    pub(super) major: u32,
    pub(super) minor: u32,
}

impl QmsVersion {
    /// Versions that are fully supported and safe to use.
    pub const SUPPORTED_VERSIONS: &[QmsVersion] = &[v1_0::VERSION];

    /// Versions that still parse and work, but should trigger a warning
    /// since they're on their way to becoming unsupported.
    pub const DEPRECATED_VERSIONS: &[QmsVersion] = &[];

    /// Versions that are known but should be rejected outright
    pub const UNSUPPORTED_VERSIONS: &[QmsVersion] = &[];

    /// Whether this version is in the fully-supported list.
    pub const fn is_supported(self) -> bool {
        Self::contains(Self::SUPPORTED_VERSIONS, self)
    }

    /// Whether this version is deprecated and should warn on use.
    pub const fn is_deprecated(self) -> bool {
        Self::contains(Self::DEPRECATED_VERSIONS, self)
    }

    /// Whether this version is explicitly unsupported and should be rejected.
    pub const fn is_unsupported(self) -> bool {
        let mut i = 0;
        while i < Self::UNSUPPORTED_VERSIONS.len() {
            let QmsVersion { major, minor } = Self::UNSUPPORTED_VERSIONS[i];
            if major == self.major && minor == self.minor {
                return true;
            }
            i += 1;
        }

        false
    }

    const fn contains(list: &[QmsVersion], version: QmsVersion) -> bool {
        let mut i = 0;
        while i < list.len() {
            let QmsVersion { major, minor } = list[i];
            if major == version.major && minor == version.minor {
                return true;
            }
            i += 1;
        }

        false
    }
}

impl FromStr for QmsVersion {
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

impl<'de> Deserialize<'de> for QmsVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VersionVisitor;

        impl Visitor<'_> for VersionVisitor {
            type Value = QmsVersion;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a schema version in the format major.minor")
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

impl fmt::Display for QmsVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}
