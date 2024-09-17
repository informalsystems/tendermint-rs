//! Support for dynamic compatibility with older protocol versions.

use core::fmt;
use core::str::FromStr;

use serde::{de::Deserializer, Deserialize, Serialize, Serializer};

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
    /// Use version 0.38 of the protocol.
    V0_38,
    // NOTE: When adding a newer version, do not forget to update:
    // - CompatMode::latest()
    // - CompatMode::from_version()
    // - impl Display for CompatMode
    // - impl FromStr for CompatMode
    // - The tests
}

impl Default for CompatMode {
    fn default() -> Self {
        CompatMode::latest()
    }
}

impl CompatMode {
    /// The latest supported version, selected by default.
    pub const fn latest() -> Self {
        Self::V0_38
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
            (0, 38) => Ok(CompatMode::V0_38),
            _ => Err(Error::unsupported_tendermint_version(version.to_string())),
        }
    }
}

impl fmt::Display for CompatMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompatMode::V0_34 => f.write_str("v0.34"),
            CompatMode::V0_37 => f.write_str("v0.37"),
            CompatMode::V0_38 => f.write_str("v0.38"),
        }
    }
}

impl FromStr for CompatMode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const VALID_COMPAT_MODES: &str = "v0.34, v0.37, v0.38";

        // Trim leading 'v', if present
        match s.trim_start_matches('v') {
            "0.34" => Ok(CompatMode::V0_34),
            "0.37" => Ok(CompatMode::V0_37),
            "0.38" => Ok(CompatMode::V0_38),
            _ => Err(Error::invalid_compat_mode(
                s.to_string(),
                VALID_COMPAT_MODES,
            )),
        }
    }
}

impl<'de> Deserialize<'de> for CompatMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de;

        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

impl Serialize for CompatMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
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
        assert_eq!(
            CompatMode::from_version(parse_version("v0.38.0")).unwrap(),
            CompatMode::V0_38
        );
        let res = CompatMode::from_version(parse_version("v0.39.0"));
        assert!(res.is_err());
        let res = CompatMode::from_version(parse_version("v1.0.0"));
        assert!(res.is_err());
        let res = CompatMode::from_version(parse_version("poobah"));
        assert!(res.is_err());
    }

    #[test]
    fn test_from_str() {
        assert_eq!("0.34".parse::<CompatMode>().unwrap(), CompatMode::V0_34);
        assert_eq!("0.37".parse::<CompatMode>().unwrap(), CompatMode::V0_37);
        assert_eq!("0.38".parse::<CompatMode>().unwrap(), CompatMode::V0_38);

        let res = "0.33".parse::<CompatMode>();
        assert!(res.is_err());
        let res = "0.39".parse::<CompatMode>();
        assert!(res.is_err());
        let res = "foobar".parse::<CompatMode>();
        assert!(res.is_err());
    }
}
