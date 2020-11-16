//! Tendermint block-related data structures.

use crate::abci::TransactionData;
use crate::bits::BitArray;
use crate::evidence::EvidenceData;
use crate::time::Time;
use crate::{serializers, version};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub header: Option<Header>,
    pub data: Option<TransactionData>,
    pub evidence: Option<EvidenceData>,
    pub last_commit: Option<Commit>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Header {
    /// basic block info
    pub version: version::Consensus,
    pub chain_id: String,
    #[serde(with = "serializers::from_str")]
    pub height: u64,
    pub time: Time,
    /// prev block info
    pub last_block_id: Option<BlockId>,
    /// hashes of block data
    ///
    /// commit from validators from the last block
    #[serde(with = "serializers::bytes::hexstring")]
    pub last_commit_hash: Vec<u8>,
    /// transactions
    #[serde(with = "serializers::bytes::hexstring")]
    pub data_hash: Vec<u8>,
    /// hashes from the app output from the prev block
    ///
    /// validators for the current block
    #[serde(with = "serializers::bytes::hexstring")]
    pub validators_hash: Vec<u8>,
    /// validators for the next block
    #[serde(with = "serializers::bytes::hexstring")]
    pub next_validators_hash: Vec<u8>,
    /// consensus params for current block
    #[serde(with = "serializers::bytes::hexstring")]
    pub consensus_hash: Vec<u8>,
    /// state after txs from the previous block
    #[serde(with = "serializers::bytes::hexstring")]
    pub app_hash: Vec<u8>,
    /// root hash of all results from the txs from the previous block
    #[serde(with = "serializers::bytes::hexstring")]
    pub last_results_hash: Vec<u8>,
    /// consensus info
    ///
    /// evidence included in the block
    #[serde(with = "serializers::bytes::hexstring")]
    pub evidence_hash: Vec<u8>,
    /// original proposer of the block
    #[serde(with = "serializers::bytes::hexstring")]
    pub proposer_address: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Commit {
    #[serde(with = "serializers::from_str")]
    pub height: i64,
    pub round: i32,
    pub block_id: Option<BlockId>,
    #[serde(with = "serializers::nullable")]
    pub signatures: Vec<CommitSig>,
    #[serde(default)]
    pub hash: Vec<u8>,
    pub bit_array: Option<BitArray>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockId {
    #[serde(with = "serializers::bytes::hexstring")]
    pub hash: Vec<u8>,
    #[serde(alias = "parts")]
    pub part_set_header: Option<PartSetHeader>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartSetHeader {
    pub total: u32,
    #[serde(with = "serializers::bytes::hexstring")]
    pub hash: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommitSig {
    pub block_id_flag: i32,
    #[serde(with = "serializers::bytes::hexstring")]
    pub validator_address: Vec<u8>,
    pub timestamp: Time,
    #[serde(with = "serializers::bytes::base64string")]
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignedHeader {
    pub header: Option<Header>,
    pub commit: Option<Commit>,
}
