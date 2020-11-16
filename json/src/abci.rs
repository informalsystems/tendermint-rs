//! Tendermint ABCI-related data structures.

use crate::crypto::ProofOps;
use crate::serializers;
use serde::{Deserialize, Serialize};

/// The response from an `abci_info` request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Info {
    pub data: String,
    pub version: String,
    #[serde(with = "serializers::from_str")]
    pub app_version: u64,
    #[serde(with = "serializers::from_str")]
    pub last_block_height: i64,
    #[serde(skip_serializing_if = "Vec::is_empty", with = "serde_bytes")]
    pub last_block_app_hash: Vec<u8>,
}

/// The response from an `abci_query` request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Query {
    /// Response code
    pub code: u32,
    /// Log value (non-deterministic)
    pub log: String,
    /// Info value (non-deterministic)
    pub info: String,
    #[serde(with = "serializers::from_str")]
    pub index: i64,
    #[serde(default, with = "serializers::bytes::base64string")]
    pub key: Vec<u8>,
    #[serde(default, with = "serializers::bytes::base64string")]
    pub value: Vec<u8>,
    /// Proof (might be explicit null)
    #[serde(alias = "proofOps")]
    pub proof_ops: Option<ProofOps>,
    /// Block height
    pub height: i64,
    #[serde(default = "String::new")]
    pub codespace: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionData {
    /// Txs that will be applied by state @ block.Height+1.
    /// NOTE: not all txs here are valid.  We're just agreeing on the order first.
    /// This means that block.AppHash does not include these txs.
    #[serde(with = "serializers::txs")]
    pub txs: Vec<Vec<u8>>,
    /// Volatile
    #[serde(default)]
    pub hash: Vec<u8>,
}
