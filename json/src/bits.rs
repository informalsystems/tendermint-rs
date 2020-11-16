//! Bit array data structure for Tendermint.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BitArray {
    pub bits: i64,
    pub elems: Vec<u64>,
}
