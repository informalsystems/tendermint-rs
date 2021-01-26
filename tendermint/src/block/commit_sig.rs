//! CommitSig within Commit

use crate::{account, Signature, Time};
use crate::{Error, Kind};
use num_traits::ToPrimitive;
use std::convert::{TryFrom, TryInto};
use tendermint_proto::types::BlockIdFlag;
use tendermint_proto::types::CommitSig as RawCommitSig;

/// CommitSig represents a signature of a validator.
/// It's a part of the Commit and can be used to reconstruct the vote set given the validator set.
#[derive(Clone, Debug, PartialEq)]
pub enum CommitSig {
    /// no vote was received from a validator.
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

impl CommitSig {
    /// Get the address of this validator if a vote was received.
    pub fn validator_address(&self) -> Option<account::Id> {
        match self {
            Self::BlockIDFlagCommit {
                validator_address, ..
            } => Some(*validator_address),
            Self::BlockIDFlagNil {
                validator_address, ..
            } => Some(*validator_address),
            _ => None,
        }
    }

    /// Whether this signature is absent (no vote was received from validator)
    pub fn is_absent(&self) -> bool {
        self == &Self::BlockIDFlagAbsent
    }

    /// Whether this signature is a commit  (validator voted for the Commit.BlockId)
    pub fn is_commit(&self) -> bool {
        matches!(self, Self::BlockIDFlagCommit { .. })
    }

    /// Whether this signature is nil (validator voted for nil)
    pub fn is_nil(&self) -> bool {
        matches!(self, Self::BlockIDFlagNil { .. })
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
                    return Err(Kind::InvalidTimestamp
                        .context("absent commitsig has non-zero timestamp")
                        .into());
                }
            }
            if !value.signature.is_empty() {
                return Err(Kind::InvalidSignature.into());
            }
            return Ok(CommitSig::BlockIDFlagAbsent);
        }
        if value.block_id_flag == BlockIdFlag::Commit.to_i32().unwrap() {
            if value.signature.is_empty() {
                return Err(Kind::InvalidSignature
                    .context("regular commitsig has no signature")
                    .into());
            }
            if value.validator_address.is_empty() {
                return Err(Kind::InvalidValidatorAddress.into());
            }
            return Ok(CommitSig::BlockIDFlagCommit {
                validator_address: value.validator_address.try_into()?,
                timestamp: value.timestamp.ok_or(Kind::NoTimestamp)?.try_into()?,
                signature: value.signature.try_into()?,
            });
        }
        if value.block_id_flag == BlockIdFlag::Nil.to_i32().unwrap() {
            if value.signature.is_empty() {
                return Err(Kind::InvalidSignature
                    .context("nil commitsig has no signature")
                    .into());
            }
            if value.validator_address.is_empty() {
                return Err(Kind::InvalidValidatorAddress.into());
            }
            return Ok(CommitSig::BlockIDFlagNil {
                validator_address: value.validator_address.try_into()?,
                timestamp: value.timestamp.ok_or(Kind::NoTimestamp)?.try_into()?,
                signature: value.signature.try_into()?,
            });
        }
        Err(Kind::BlockIdFlag.into())
    }
}

impl From<CommitSig> for RawCommitSig {
    fn from(commit: CommitSig) -> RawCommitSig {
        match commit {
            CommitSig::BlockIDFlagAbsent => RawCommitSig {
                block_id_flag: BlockIdFlag::Absent.to_i32().unwrap(),
                validator_address: Vec::new(),
                timestamp: None,
                signature: Vec::new(),
            },
            CommitSig::BlockIDFlagNil {
                validator_address,
                timestamp,
                signature,
            } => RawCommitSig {
                block_id_flag: BlockIdFlag::Nil.to_i32().unwrap(),
                validator_address: validator_address.into(),
                timestamp: Some(timestamp.into()),
                signature: signature.into(),
            },
            CommitSig::BlockIDFlagCommit {
                validator_address,
                timestamp,
                signature,
            } => RawCommitSig {
                block_id_flag: BlockIdFlag::Commit.to_i32().unwrap(),
                validator_address: validator_address.into(),
                timestamp: Some(timestamp.into()),
                signature: signature.into(),
            },
        }
    }
}
