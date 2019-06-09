//! Chain-specific key configuration

use serde::Deserialize;
use tendermint::TendermintKey;

/// Options for how keys for this chain are represented
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Format {
    /// Use the Bech32 serialization format with the given key prefixes
    #[serde(rename = "bech32")]
    Bech32 {
        /// Prefix to use for Account keys
        account_key_prefix: String,

        /// Prefix to use for Consensus keys
        consensus_key_prefix: String,
    },

    /// Hex is a baseline representation
    #[serde(rename = "hex")]
    Hex,
}

impl Format {
    /// Serialize a `TendermintKey` according to chain-specific rules
    pub fn serialize(&self, public_key: TendermintKey) -> String {
        match self {
            Format::Bech32 {
                account_key_prefix,
                consensus_key_prefix,
            } => match public_key {
                TendermintKey::AccountKey(pk) => pk.to_bech32(account_key_prefix),
                TendermintKey::ConsensusKey(pk) => pk.to_bech32(consensus_key_prefix),
            },
            Format::Hex => public_key.to_hex(),
        }
    }
}
