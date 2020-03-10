use crate::{account, Time, Signature};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum BlockIDFlag {
    /// BlockIDFlagAbsent - no vote was received from a validator.
    BlockIDFlagAbsent = 0x01,
    /// BlockIDFlagCommit - voted for the Commit.BlockID.
    BlockIDFlagCommit = 0x02,
    /// BlockIDFlagNil - voted for nil.
    BlockIDFlagNil = 0x03,
}

/// CommitSig represents a signature of a validator.
/// It's a part of the Commit and can be used to reconstruct the vote set given the validator set.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CommitSig {
    /// Block ID FLag
    pub block_id_flag: BlockIDFlag,

    /// Validator address
    pub validator_address: account::Id,

    /// Timestamp
    pub timestamp: Time,

    /// Signature
    pub signature: Signature,
}