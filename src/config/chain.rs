//! Chain configuration

use crate::chain;

/// Chain configuration
#[derive(Clone, Deserialize, Debug)]
pub struct ChainConfig {
    /// Chain ID of this Tendermint network/chain
    pub id: chain::Id,

    /// Key format configuration
    pub key_format: chain::key::Format,
}
