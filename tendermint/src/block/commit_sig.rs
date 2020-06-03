//! CommitSig within Commit

use crate::serializers::BlockIDFlag;
use crate::serializers::RawCommitSig;
use crate::{account, Signature, Time};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// CommitSig represents a signature of a validator.
/// It's a part of the Commit and can be used to reconstruct the vote set given the validator set.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(try_from = "RawCommitSig", into = "RawCommitSig")]
pub enum CommitSig {
    /// no vote was received from a validator.
    // Todo: https://github.com/informalsystems/tendermint-rs/issues/260 - CommitSig validator address missing in Absent vote
    BlockIDFlagAbsent,
    /// voted for the Commit.BlockID.
    BlockIDFlagCommit {
        /// Validator address
        validator_address: account::Id,
        /// Timestamp of vote
        timestamp: Time,
        /// Signature of vote
        signature: Signature,
    },
    /// voted for nil.
    BlockIDFlagNil {
        /// Validator address
        validator_address: account::Id,
        /// Timestamp of vote
        timestamp: Time,
        /// Signature of vote
        signature: Signature,
    },
}

// Todo: https://github.com/informalsystems/tendermint-rs/issues/259 - CommitSig Timestamp can be zero time
// Todo: https://github.com/informalsystems/tendermint-rs/issues/260 - CommitSig validator address missing in Absent vote
impl TryFrom<RawCommitSig> for CommitSig {
    type Error = &'static str;

    fn try_from(value: RawCommitSig) -> Result<Self, Self::Error> {
        match value.block_id_flag {
            BlockIDFlag::Absent => {
                if value.timestamp.is_some()
                    && value.timestamp.unwrap()
                        != Time::parse_from_rfc3339("0001-01-01T00:00:00Z").unwrap()
                {
                    return Err("timestamp is present for BlockIDFlagAbsent CommitSig");
                }
                if value.signature.is_some() {
                    return Err("signature is present for BlockIDFlagAbsent CommitSig");
                }
                Ok(CommitSig::BlockIDFlagAbsent)
            }
            BlockIDFlag::Commit => {
                if value.timestamp.is_none() {
                    Err("timestamp is missing for BlockIDFlagCommit CommitSig")
                } else if value.signature.is_none() {
                    Err("signature is missing for BlockIDFlagCommit CommitSig")
                } else if value.validator_address.is_none() {
                    Err("validator_address is missing for BlockIDFlagCommit CommitSig")
                } else {
                    Ok(CommitSig::BlockIDFlagCommit {
                        validator_address: value.validator_address.unwrap(),
                        timestamp: value.timestamp.unwrap(),
                        signature: value.signature.unwrap(),
                    })
                }
            }
            BlockIDFlag::Nil => {
                if value.timestamp.is_none() {
                    Err("timestamp is missing for BlockIDFlagNil CommitSig")
                } else if value.signature.is_none() {
                    Err("signature is missing for BlockIDFlagNil CommitSig")
                } else if value.validator_address.is_none() {
                    Err("validator_address is missing for BlockIDFlagNil CommitSig")
                } else {
                    Ok(CommitSig::BlockIDFlagNil {
                        validator_address: value.validator_address.unwrap(),
                        timestamp: value.timestamp.unwrap(),
                        signature: value.signature.unwrap(),
                    })
                }
            }
        }
    }
}

impl From<CommitSig> for RawCommitSig {
    fn from(commit: CommitSig) -> RawCommitSig {
        match commit {
            CommitSig::BlockIDFlagAbsent => RawCommitSig {
                block_id_flag: BlockIDFlag::Absent,
                validator_address: None,
                timestamp: None,
                signature: None,
            },
            CommitSig::BlockIDFlagNil {
                validator_address,
                timestamp,
                signature,
            } => RawCommitSig {
                block_id_flag: BlockIDFlag::Nil,
                validator_address: Some(validator_address),
                timestamp: Some(timestamp),
                signature: Some(signature),
            },
            CommitSig::BlockIDFlagCommit {
                validator_address,
                timestamp,
                signature,
            } => RawCommitSig {
                block_id_flag: BlockIDFlag::Commit,
                validator_address: Some(validator_address),
                timestamp: Some(timestamp),
                signature: Some(signature),
            },
        }
    }
}
