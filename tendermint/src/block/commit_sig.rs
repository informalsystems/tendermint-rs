//! CommitSig within Commit

use crate::error::Error;
use crate::prelude::*;
use crate::{account, Signature, Time};
use core::convert::{TryFrom, TryInto};
use num_traits::ToPrimitive;
use tendermint_proto::types::BlockIdFlag;
use tendermint_proto::types::CommitSig as RawCommitSig;

/// CommitSig represents a signature of a validator.
/// It's a part of the Commit and can be used to reconstruct the vote set given the validator set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommitSig {
    /// no vote was received from a validator.
    BlockIdFlagAbsent,
    /// voted for the Commit.BlockID.
    BlockIdFlagCommit {
        /// Validator address
        validator_address: account::Id,
        /// Timestamp of vote
        timestamp: Time,
        /// Signature of vote
        signature: Option<Signature>,
    },
    /// voted for nil.
    BlockIdFlagNil {
        /// Validator address
        validator_address: account::Id,
        /// Timestamp of vote
        timestamp: Time,
        /// Signature of vote
        signature: Option<Signature>,
    },
}

impl CommitSig {
    /// Get the address of this validator if a vote was received.
    pub fn validator_address(&self) -> Option<account::Id> {
        match self {
            Self::BlockIdFlagCommit {
                validator_address, ..
            } => Some(*validator_address),
            Self::BlockIdFlagNil {
                validator_address, ..
            } => Some(*validator_address),
            _ => None,
        }
    }

    /// Whether this signature is absent (no vote was received from validator)
    pub fn is_absent(&self) -> bool {
        self == &Self::BlockIdFlagAbsent
    }

    /// Whether this signature is a commit  (validator voted for the Commit.BlockId)
    pub fn is_commit(&self) -> bool {
        matches!(self, Self::BlockIdFlagCommit { .. })
    }

    /// Whether this signature is nil (validator voted for nil)
    pub fn is_nil(&self) -> bool {
        matches!(self, Self::BlockIdFlagNil { .. })
    }
}

// Todo: https://github.com/informalsystems/tendermint-rs/issues/259 - CommitSig Timestamp can be zero time
// Todo: https://github.com/informalsystems/tendermint-rs/issues/260 - CommitSig validator address missing in Absent vote
impl TryFrom<RawCommitSig> for CommitSig {
    type Error = Error;

    fn try_from(value: RawCommitSig) -> Result<Self, Self::Error> {
        if value.block_id_flag == BlockIdFlag::Absent.to_i32().unwrap() {
            if value.timestamp.is_some() {
                let timestamp = value.timestamp.unwrap();
                // 0001-01-01T00:00:00.000Z translates to EPOCH-62135596800 seconds
                if timestamp.nanos != 0 || timestamp.seconds != -62135596800 {
                    return Err(Error::invalid_timestamp(
                        "absent commitsig has non-zero timestamp".to_string(),
                    ));
                }
            }

            if !value.signature.is_empty() {
                return Err(Error::invalid_signature(
                    "expected empty signature for absent commitsig".to_string(),
                ));
            }

            return Ok(CommitSig::BlockIdFlagAbsent);
        }

        if value.block_id_flag == BlockIdFlag::Commit.to_i32().unwrap() {
            if value.signature.is_empty() {
                return Err(Error::invalid_signature(
                    "expected non-empty signature for regular commitsig".to_string(),
                ));
            }

            if value.validator_address.is_empty() {
                return Err(Error::invalid_validator_address());
            }

            let timestamp = value
                .timestamp
                .ok_or_else(Error::missing_timestamp)?
                .try_into()?;

            return Ok(CommitSig::BlockIdFlagCommit {
                validator_address: value.validator_address.try_into()?,
                timestamp,
                signature: Signature::new(value.signature)?,
            });
        }
        if value.block_id_flag == BlockIdFlag::Nil.to_i32().unwrap() {
            if value.signature.is_empty() {
                return Err(Error::invalid_signature(
                    "nil commitsig has no signature".to_string(),
                ));
            }
            if value.validator_address.is_empty() {
                return Err(Error::invalid_validator_address());
            }
            return Ok(CommitSig::BlockIdFlagNil {
                validator_address: value.validator_address.try_into()?,
                timestamp: value
                    .timestamp
                    .ok_or_else(Error::missing_timestamp)?
                    .try_into()?,
                signature: Signature::new(value.signature)?,
            });
        }
        Err(Error::block_id_flag())
    }
}

impl From<CommitSig> for RawCommitSig {
    fn from(commit: CommitSig) -> RawCommitSig {
        match commit {
            CommitSig::BlockIdFlagAbsent => RawCommitSig {
                block_id_flag: BlockIdFlag::Absent.to_i32().unwrap(),
                validator_address: Vec::new(),
                timestamp: None,
                signature: Vec::new(),
            },
            CommitSig::BlockIdFlagNil {
                validator_address,
                timestamp,
                signature,
            } => RawCommitSig {
                block_id_flag: BlockIdFlag::Nil.to_i32().unwrap(),
                validator_address: validator_address.into(),
                timestamp: Some(timestamp.into()),
                signature: signature.map(|s| s.to_bytes()).unwrap_or_default(),
            },
            CommitSig::BlockIdFlagCommit {
                validator_address,
                timestamp,
                signature,
            } => RawCommitSig {
                block_id_flag: BlockIdFlag::Commit.to_i32().unwrap(),
                validator_address: validator_address.into(),
                timestamp: Some(timestamp.into()),
                signature: signature.map(|s| s.to_bytes()).unwrap_or_default(),
            },
        }
    }
}
