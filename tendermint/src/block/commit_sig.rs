//! CommitSig within Commit

use crate::serializers::BlockIDFlag;
use crate::serializers::RawCommitSig;
use crate::{account, Signature, Time};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// CommitSig represents a signature of a validator.
/// It's a part of the Commit and can be used to reconstruct the vote set given the validator set.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RawCommitSig")]
pub enum CommitSig {
    /// no vote was received from a validator.
    BlockIDFlagAbsent {
        /// Validator address
        validator_address: account::Id,
    },
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

/// CommitSig implementation
impl CommitSig {
    /// Helper: Extract validator address, since it's always present (according to ADR-025)
    pub fn validator_address(&self) -> &account::Id {
        match &self {
            CommitSig::BlockIDFlagAbsent { validator_address } => validator_address,
            CommitSig::BlockIDFlagCommit {
                validator_address, ..
            } => validator_address,
            CommitSig::BlockIDFlagNil {
                validator_address, ..
            } => validator_address,
        }
    }
}

impl TryFrom<RawCommitSig> for CommitSig {
    type Error = &'static str;

    fn try_from(value: RawCommitSig) -> Result<Self, Self::Error> {
        // Validate CommitSig (strict)
        match value.block_id_flag {
            BlockIDFlag::BlockIDFlagAbsent => {
                if value.timestamp.is_some() {
                    return Err("timestamp is present for BlockIDFlagAbsent CommitSig");
                }
                if value.signature.is_some() {
                    return Err("signature is present for BlockIDFlagAbsent CommitSig");
                }
                Ok(CommitSig::BlockIDFlagAbsent {
                    validator_address: value.validator_address,
                })
            }
            BlockIDFlag::BlockIDFlagCommit => {
                if value.timestamp.is_none() {
                    Err("timestamp is null for BlockIDFlagCommit CommitSig")
                } else if value.signature.is_none() {
                    Err("signature is null for BlockIDFlagCommit CommitSig")
                } else {
                    Ok(CommitSig::BlockIDFlagCommit {
                        validator_address: value.validator_address,
                        timestamp: value.timestamp.unwrap(),
                        signature: value.signature.unwrap(),
                    })
                }
            }
            BlockIDFlag::BlockIDFlagNil => {
                if value.timestamp.is_none() {
                    Err("timestamp is null for BlockIDFlagNil CommitSig")
                } else if value.signature.is_none() {
                    Err("signature is null for BlockIDFlagNil CommitSig")
                } else {
                    Ok(CommitSig::BlockIDFlagNil {
                        validator_address: value.validator_address,
                        timestamp: value.timestamp.unwrap(),
                        signature: value.signature.unwrap(),
                    })
                }
            }
        }
    }
}
