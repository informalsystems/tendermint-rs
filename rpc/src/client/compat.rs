//! Support for dynamic compatibility with older protocol versions.

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
