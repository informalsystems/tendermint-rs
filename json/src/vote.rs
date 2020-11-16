//! Vote-related data structures for Tendermint.

use crate::block::BlockId;
use crate::serializers;
use crate::time::Time;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vote {
    pub r#type: i32,
    #[serde(with = "serializers::from_str")]
    pub height: i64,
    #[serde(with = "serializers::from_str")]
    pub round: i32,
    /// zero if vote is nil.
    pub block_id: Option<BlockId>,
    pub timestamp: Time,
    #[serde(with = "serializers::bytes::hexstring")]
    pub validator_address: Vec<u8>,
    #[serde(with = "serializers::from_str")]
    pub validator_index: i32,
    #[serde(with = "serializers::bytes::base64string")]
    pub signature: Vec<u8>,
}
