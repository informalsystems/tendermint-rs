//! Block headers

use crate::{account, block, chain, Hash, Timestamp};
#[cfg(feature = "serde")]
use {
    crate::serializers,
    serde::{Deserialize, Serialize},
};

/// Block header
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct Header {
    /// Header version
    pub version: Version,

    /// Chain ID
    pub chain_id: chain::Id,

    /// Current block height
    pub height: block::Height,

    /// Current timestamp
    pub time: Timestamp,

    /// Number of transactions in block
    #[cfg_attr(
        feature = "serde",
        serde(
            serialize_with = "serializers::serialize_u64",
            deserialize_with = "serializers::parse_u64"
        )
    )]
    pub num_txs: u64,

    /// Total number of transactions
    #[cfg_attr(
        feature = "serde",
        serde(
            serialize_with = "serializers::serialize_u64",
            deserialize_with = "serializers::parse_u64"
        )
    )]
    pub total_txs: u64,

    /// Last block ID
    pub last_block_id: block::Id,

    /// Last commit hash
    pub last_commit_hash: Hash,

    /// Data hash
    pub data_hash: Hash,

    /// Validators hash
    pub validators_hash: Hash,

    /// Next validators hash
    pub next_validators_hash: Hash,

    /// Consensus hash
    pub consensus_hash: Hash,

    /// App hash
    pub app_hash: Hash,

    /// Last results hash
    pub last_results_hash: Hash,

    /// Evidence hash
    pub evidence_hash: Hash,

    /// Proposer address
    pub proposer_address: account::Id,
}

/// Block header versions
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct Version {
    /// Block version
    #[cfg_attr(
        feature = "serde",
        serde(
            serialize_with = "serializers::serialize_u64",
            deserialize_with = "serializers::parse_u64"
        )
    )]
    pub block: u64,

    /// App version
    #[cfg_attr(
        feature = "serde",
        serde(
            serialize_with = "serializers::serialize_u64",
            deserialize_with = "serializers::parse_u64"
        )
    )]
    pub app: u64,
}
