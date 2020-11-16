//! Cryptography-related data structures for Tendermint.

use serde::{Deserialize, Serialize};

/// ProofOp defines an operation used for calculating Merkle root
/// The data could be arbitrary format, providing necessary data
/// for example neighbouring node hash.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProofOp {
    pub r#type: String,
    pub key: Vec<u8>,
    pub data: Vec<u8>,
}

/// ProofOps is Merkle proof defined by the list of ProofOps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProofOps {
    pub ops: Vec<ProofOp>,
}
