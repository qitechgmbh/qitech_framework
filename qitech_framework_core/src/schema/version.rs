use std::fmt;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version {
    pub(super) major: u32,
    pub(super) minor: u32,
}

impl Version {
    /// Versions that are fully supported and safe to use.
    pub const SUPPORTED_VERSIONS: &[Version] = &[Version { major: 1, minor: 0 }];

    /// Versions that still parse and work, but should trigger a warning
    /// since they're on their way to becoming unsupported.
    pub const DEPRECATED_VERSIONS: &[Version] = &[];

    /// Versions that are known but should be rejected outright
    pub const UNSUPPORTED_VERSIONS: &[Version] = &[];

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
        Self::contains(Self::UNSUPPORTED_VERSIONS, self)
    }

    // `PartialEq::eq` isn't callable in const fn, so compare fields by hand.
    const fn contains(mut list: &[Version], version: Version) -> bool {
        while let [head, rest @ ..] = list {
            if head.major == version.major && head.minor == version.minor {
                return true;
            }
            list = rest;
        }

        false
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}
