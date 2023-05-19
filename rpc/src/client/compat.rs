//! Support for dynamic compatibility with older protocol versions.

use core::fmt;

use tendermint::Version;

use crate::prelude::*;
use crate::Error;

/// Protocol compatibility mode for a Tendermint RPC client.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CompatMode {
    /// Use version 0.34 of the protocol.
    V0_34,
    /// Use version 0.37 of the protocol.
    V0_37,
}

impl Default for CompatMode {
    fn default() -> Self {
        CompatMode::latest()
    }
}

impl CompatMode {
    /// The latest supported version, selected by default.
    pub const fn latest() -> Self {
        Self::V0_37
    }

    /// Parse the Tendermint version string to determine
    /// the compatibility mode.
    ///
    /// The version can be obtained by querying the `/status` endpoint.
    /// The request and response format of this endpoint is currently the same
    /// for all supported RPC dialects, so such a request can be performed
    /// before the required compatibility mode is settled upon.
    ///
    /// Note that this is not fail-proof: the version is reported for a particular
    /// client connection, so any other connections to the same URL might not
    /// be handled by the same server. In the future, the RPC protocol should
    /// follow versioning practices designed to avoid ambiguities with
    /// message formats.
    pub fn from_version(tendermint_version: Version) -> Result<CompatMode, Error> {
        let raw_version: String = tendermint_version.into();
        let version = semver::Version::parse(raw_version.trim_start_matches('v'))
            .map_err(|_| Error::invalid_tendermint_version(raw_version))?;

        match (version.major, version.minor) {
            (0, 34) => Ok(CompatMode::V0_34),
            (0, 37) => Ok(CompatMode::V0_37),
            (0, 38) => Ok(CompatMode::V0_37),
            _ => Err(Error::unsupported_tendermint_version(version.to_string())),
        }
    }
}

impl fmt::Display for CompatMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompatMode::V0_34 => f.write_str("v0.34"),
            CompatMode::V0_37 => f.write_str("v0.37"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CompatMode;
    use crate::prelude::*;
    use tendermint::Version;

    fn parse_version(s: &str) -> Version {
        let json = format!("\"{s}\"");
        serde_json::from_str(&json).unwrap()
    }

    #[test]
    fn test_parse_version_for_compat_mode() {
        assert_eq!(
            CompatMode::from_version(parse_version("v0.34.16")).unwrap(),
            CompatMode::V0_34
        );
        assert_eq!(
            CompatMode::from_version(parse_version("v0.37.0-pre1")).unwrap(),
            CompatMode::V0_37
        );
        assert_eq!(
            CompatMode::from_version(parse_version("v0.37.0")).unwrap(),
            CompatMode::V0_37
        );
        let res = CompatMode::from_version(parse_version("v0.38.0"));
        assert!(res.is_err());
        let res = CompatMode::from_version(parse_version("v1.0.0"));
        assert!(res.is_err());
        let res = CompatMode::from_version(parse_version("poobah"));
        assert!(res.is_err());
    }
}
