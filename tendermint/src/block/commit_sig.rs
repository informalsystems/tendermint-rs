//! CommitSig within Commit

use tendermint_proto::google::protobuf::Timestamp;

use crate::{account, prelude::*, Signature, Time};

/// The special zero timestamp is to be used for absent votes,
/// where there is no timestamp to speak of.
///
/// It is not the standard UNIX epoch at 0 seconds, ie. 1970-01-01 00:00:00 UTC,
/// but a custom Tendermint-specific one for 1-1-1 00:00:00 UTC
///
/// From the corresponding Tendermint `Time` struct:
///
/// The zero value for a Time is defined to be
/// January 1, year 1, 00:00:00.000000000 UTC
/// which (1) looks like a zero, or as close as you can get in a date
/// (1-1-1 00:00:00 UTC), (2) is unlikely enough to arise in practice to
/// be a suitable "not set" sentinel, unlike Jan 1 1970, and (3) has a
/// non-negative year even in time zones west of UTC, unlike 1-1-0
/// 00:00:00 UTC, which would be 12-31-(-1) 19:00:00 in New York.
const ZERO_TIMESTAMP: Timestamp = Timestamp {
    seconds: -62135596800,
    nanos: 0,
};

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

tendermint_pb_modules! {
    use super::{CommitSig, ZERO_TIMESTAMP};
    use crate::{error::Error, prelude::*, Signature};

    use num_traits::ToPrimitive;
    use pb::types::{BlockIdFlag, CommitSig as RawCommitSig};

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
                    timestamp: Some(ZERO_TIMESTAMP),
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
                    signature: signature.map(|s| s.into_bytes()).unwrap_or_default(),
                },
                CommitSig::BlockIdFlagCommit {
                    validator_address,
                    timestamp,
                    signature,
                } => RawCommitSig {
                    block_id_flag: BlockIdFlag::Commit.to_i32().unwrap(),
                    validator_address: validator_address.into(),
                    timestamp: Some(timestamp.into()),
                    signature: signature.map(|s| s.into_bytes()).unwrap_or_default(),
                },
            }
        }
    }

    #[test]
    #[cfg(test)]
    fn test_block_id_flag_absent_serialization() {
        let absent = CommitSig::BlockIdFlagAbsent;
        let raw_absent = RawCommitSig::from(absent);
        let expected = r#"{"block_id_flag":1,"validator_address":"","timestamp":"0001-01-01T00:00:00Z","signature":""}"#;
        let output = serde_json::to_string(&raw_absent).unwrap();
        assert_eq!(expected, &output);
    }

    #[test]
    #[cfg(test)]
    fn test_block_id_flag_absent_deserialization() {
        let json = r#"{"block_id_flag":1,"validator_address":"","timestamp":"0001-01-01T00:00:00Z","signature":""}"#;
        let raw_commit_sg = serde_json::from_str::<RawCommitSig>(json).unwrap();
        let commit_sig = CommitSig::try_from(raw_commit_sg).unwrap();
        assert_eq!(commit_sig, CommitSig::BlockIdFlagAbsent);
    }
}
