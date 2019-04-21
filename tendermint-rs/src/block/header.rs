//! Block headers

use crate::{account, block, chain, Hash, Time};
#[cfg(feature = "serde")]
use {
    crate::serializers,
    serde::{Deserialize, Serialize},
};

/// Block `Header` values contain metadata about the block and about the
/// consensus, as well as commitments to the data in the current block, the
/// previous block, and the results returned by the application.
///
/// <https://github.com/tendermint/tendermint/blob/master/docs/spec/blockchain/blockchain.md#header>
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
    pub time: Time,

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

    /// Previous block info
    pub last_block_id: block::Id,

    /// Commit from validators from the last block
    pub last_commit_hash: Hash,

    /// Merkle root of transaction hashes
    pub data_hash: Hash,

    /// Validators for the current block
    pub validators_hash: Hash,

    /// Validators for the next block
    pub next_validators_hash: Hash,

    /// Consensus params for the current block
    pub consensus_hash: Hash,

    /// State after txs from the previous block
    pub app_hash: Hash,

    /// Root hash of all results from the txs from the previous block
    pub last_results_hash: Hash,

    /// Hash of evidence included in the block
    pub evidence_hash: Hash,

    /// Original proposer of the block
    pub proposer_address: account::Id,
}

/// `Version` contains the protocol version for the blockchain and the
/// application.
///
/// <https://github.com/tendermint/tendermint/blob/master/docs/spec/blockchain/blockchain.md#version>
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
