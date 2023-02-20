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
    let raw_version: String = status.node_info.version.into();
    let tm_version = semver::Version::parse(&raw_version)
        .map_err(|_| Error::invalid_tendermint_version(raw_version))?;
    match (tm_version.major, tm_version.minor) {
        (0, 34) => Ok(CompatMode::V0_34),
        (0, 37) => Ok(CompatMode::Latest),
        _ => Err(Error::unsupported_tendermint_version(
            tm_version.to_string(),
        )),
    }
}
