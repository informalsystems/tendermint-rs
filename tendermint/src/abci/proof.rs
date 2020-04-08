//! ABCI Merkle proofs

use serde::{Deserialize, Serialize};

/// Proof is Merkle proof defined by the list of ProofOps
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Proof {
    /// The list of ProofOps
    pub ops: Vec<ProofOp>,
}

/// ProofOp defines an operation used for calculating Merkle root
/// The data could be arbitrary format, providing necessary data
/// for example neighbouring node hash
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ProofOp {
    /// Type of the ProofOp
    pub field_type: String,
    /// Key of the ProofOp
    pub key: Vec<u8>,
    /// Actual data
    pub data: Vec<u8>,
}
