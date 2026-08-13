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
        let mut i = 0;
        while i < Self::UNSUPPORTED_VERSIONS.len() {
            let Version { major, minor } = Self::UNSUPPORTED_VERSIONS[i];
            if major == self.major && minor == self.minor {
                return true;
            }
            i += 1;
        }

        false
    }

    const fn contains(list: &[Version], version: Version) -> bool {
        let mut i = 0;
        while i < list.len() {
            let Version { major, minor } = list[i];
            if major == version.major && minor == version.minor {
                return true;
            }
            i += 1;
        }

        false
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}
