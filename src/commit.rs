//! Commit messages used for updating chain state

use crate::{account, block, Signature, Timestamp};
#[cfg(feature = "serde")]
use {
    crate::serializers,
    serde::{Deserialize, Serialize},
};

/// Last commit
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct LastCommit {
    /// Block ID of the last commit
    pub block_id: block::Id,

    /// Precommits
    pub precommits: Vec<Option<Precommit>>,
}

/// Precommits
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct Precommit {
    /// Type of precommit
    // TODO(tarcieri): use an `enum` for this?
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub precommit_type: u64,

    /// Block height
    pub height: block::Height,

    /// Round
    #[cfg_attr(
        feature = "serde",
        serde(
            serialize_with = "serializers::serialize_u64",
            deserialize_with = "serializers::parse_u64"
        )
    )]
    pub round: u64,

    /// Block ID
    pub block_id: block::Id,

    /// Timestamp
    pub timestamp: Timestamp,

    /// Validator address
    pub validator_address: account::Id,

    /// Validator index
    #[cfg_attr(
        feature = "serde",
        serde(
            serialize_with = "serializers::serialize_u64",
            deserialize_with = "serializers::parse_u64"
        )
    )]
    pub validator_index: u64,

    /// Signature
    pub signature: Signature,
}
