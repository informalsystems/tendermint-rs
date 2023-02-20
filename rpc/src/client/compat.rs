//! Support for dynamic compatibility with older protocol versions.

use super::Client;
use crate::prelude::*;
use crate::Error;

/// Protocol compatibility mode for a Tendermint RPC client.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CompatMode {
    /// Use the latest version of the protocol (v0.37 in this release).
    Latest,
    /// Use v0.34 version of the protocol.
    V0_34,
}

impl Default for CompatMode {
    fn default() -> Self {
        CompatMode::Latest
    }
}

impl CompatMode {
    fn from_tendermint_version(raw_version: String) -> Result<CompatMode, Error> {
        let version = semver::Version::parse(&raw_version)
            .map_err(|_| Error::invalid_tendermint_version(raw_version))?;
        match (version.major, version.minor) {
            (0, 34) => Ok(CompatMode::V0_34),
            (0, 37) => Ok(CompatMode::Latest),
            _ => Err(Error::unsupported_tendermint_version(version.to_string())),
        }
    }
}

/// Queries the `/status` RPC endpoint to discover which compatibility mode
/// to use for the node that this client connects to.
///
/// Note that this is not fail-proof: the version is reported for a particular
/// client connection, so any other connections to the same URL might not
/// be handled by the same server. In the future, the RPC protocol should
/// follow versioning practices designed to avoid ambiguities with
/// message formats.
pub async fn discover<C>(client: &C) -> Result<CompatMode, Error>
where
    C: Client + Send + Sync,
{
    let status = client.status().await?;
    CompatMode::from_tendermint_version(status.node_info.version.into())
}

#[cfg(test)]
mod tests {
    use super::CompatMode;

    #[test]
    fn test_parse_version_for_compat_mode() {
        assert_eq!(
            CompatMode::from_tendermint_version("0.34.16".into()).unwrap(),
            CompatMode::V0_34
        );
        assert_eq!(
            CompatMode::from_tendermint_version("0.37.0-pre1".into()).unwrap(),
            CompatMode::Latest
        );
        assert_eq!(
            CompatMode::from_tendermint_version("0.37.0".into()).unwrap(),
            CompatMode::Latest
        );
        let res = CompatMode::from_tendermint_version("0.38.0".into());
        assert!(res.is_err());
        let res = CompatMode::from_tendermint_version("1.0.0".into());
        assert!(res.is_err());
        let res = CompatMode::from_tendermint_version("poobah".into());
        assert!(res.is_err());
    }
}
