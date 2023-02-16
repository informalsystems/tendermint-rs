//! ABCI-specific data types used in requests and responses.
//!
//! These types have changes from the core data structures to better accommodate
//! ABCI applications.
//!
//! [ABCI documentation](https://docs.tendermint.com/master/spec/abci/abci.html#data-types)

use bytes::Bytes;

use crate::{block, prelude::*, vote, Time};

/// A validator address with voting power.
///
/// [ABCI documentation](https://docs.tendermint.com/master/spec/abci/abci.html#validator)
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Validator {
    /// The validator's address (the first 20 bytes of `SHA256(public_key)`).
    pub address: [u8; 20],
    /// The voting power of the validator.
    pub power: vote::Power,
}

/// Information about a whether a validator signed the last block.
///
/// [ABCI documentation](https://docs.tendermint.com/master/spec/abci/abci.html#voteinfo)
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct VoteInfo {
    /// Identifies the validator.
    pub validator: Validator,
    /// Whether or not the validator signed the last block.
    pub signed_last_block: bool,
}

/// The possible kinds of [`Misbehavior`].
///
/// Note: the
/// [ABCI documentation](https://github.com/tendermint/tendermint/blob/main/spec/abci/abci++_methods.md#misbehaviortype)
/// calls this `MisbehaviorType`, but we follow the Rust convention and name it `MisbehaviorKind`
/// to avoid confusion with Rust types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum MisbehaviorKind {
    /// Unknown evidence type (proto default value).
    Unknown = 0,
    /// Evidence that the validator voted for two different blocks in the same
    /// round of the same height.
    DuplicateVote = 1,
    /// Evidence that a validator attacked a light client.
    LightClientAttack = 2,
}

/// Evidence of validator misbehavior.
///
/// [ABCI documentation](https://docs.tendermint.com/master/spec/abci/abci.html#evidence)
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Misbehavior {
    /// The kind of evidence.
    ///
    /// Note: this field is called `type` in the protobuf, but we call it `kind`
    /// to avoid the Rust keyword.
    pub kind: MisbehaviorKind,
    /// The offending validator.
    pub validator: Validator,
    /// The height when the offense occurred.
    pub height: block::Height,
    /// The corresponding time when the offense occurred.
    pub time: Time,
    /// Total voting power of the validator set at `height`.
    ///
    /// This is included in case the ABCI application does not store historical
    /// validators, cf.
    /// [#4581](https://github.com/tendermint/tendermint/issues/4581)
    pub total_voting_power: vote::Power,
}

/// Information on a block commit.
///
/// [ABCI documentation](https://github.com/tendermint/tendermint/blob/main/spec/abci/abci++_methods.md#extendedcommitinfo)
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CommitInfo {
    /// The commit round.
    ///
    /// Reflects the total number of rounds it took to come to consensus for the
    /// current block.
    pub round: block::Round,
    /// The list of validator addresses in the last validator set, with their
    /// voting power and whether or not they signed a vote.
    pub votes: Vec<VoteInfo>,
}

/// Used for state sync snapshots.
///
/// When sent across the network, a `Snapshot` can be at most 4 MB.
///
/// [ABCI documentation](https://docs.tendermint.com/master/spec/abci/abci.html#snapshot)
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Snapshot {
    /// The height at which the snapshot was taken
    pub height: block::Height,
    /// The application-specific snapshot format identifier.
    ///
    /// This allows applications to version their snapshot data format and make
    /// backwards-incompatible changes. Tendermint does not interpret this field.
    pub format: u32,
    /// The number of chunks in the snapshot. Must be at least 1.
    pub chunks: u32,
    /// An arbitrary snapshot hash.
    ///
    /// This hash must be equal only for identical snapshots across nodes.
    /// Tendermint does not interpret the hash, only compares it with other
    /// hashes.
    pub hash: Bytes,
    /// Arbitrary application metadata, e.g., chunk hashes or other verification data.
    pub metadata: Bytes,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

mod v0_34 {
    use super::{CommitInfo, Misbehavior, MisbehaviorKind, Snapshot, Validator, VoteInfo};
    use crate::{prelude::*, Error};
    use tendermint_proto::v0_34::abci as pb;
    use tendermint_proto::Protobuf;

    use bytes::Bytes;

    impl From<Validator> for pb::Validator {
        fn from(v: Validator) -> Self {
            Self {
                address: Bytes::copy_from_slice(&v.address[..]),
                power: v.power.into(),
            }
        }
    }

    impl TryFrom<pb::Validator> for Validator {
        type Error = Error;

        fn try_from(vu: pb::Validator) -> Result<Self, Self::Error> {
            let address = if vu.address.len() == 20 {
                let mut bytes = [0u8; 20];
                bytes.copy_from_slice(&vu.address);
                bytes
            } else {
                return Err(Error::invalid_account_id_length());
            };

            Ok(Self {
                address,
                power: vu.power.try_into()?,
            })
        }
    }

    impl Protobuf<pb::Validator> for Validator {}

    impl From<VoteInfo> for pb::VoteInfo {
        fn from(vi: VoteInfo) -> Self {
            Self {
                validator: Some(vi.validator.into()),
                signed_last_block: vi.signed_last_block,
            }
        }
    }

    impl TryFrom<pb::VoteInfo> for VoteInfo {
        type Error = Error;

        fn try_from(vi: pb::VoteInfo) -> Result<Self, Self::Error> {
            Ok(Self {
                validator: vi
                    .validator
                    .ok_or_else(Error::missing_validator)?
                    .try_into()?,
                signed_last_block: vi.signed_last_block,
            })
        }
    }

    impl Protobuf<pb::VoteInfo> for VoteInfo {}

    impl From<Misbehavior> for pb::Evidence {
        fn from(evidence: Misbehavior) -> Self {
            Self {
                r#type: evidence.kind as i32,
                validator: Some(evidence.validator.into()),
                height: evidence.height.into(),
                time: Some(evidence.time.into()),
                total_voting_power: evidence.total_voting_power.into(),
            }
        }
    }

    impl TryFrom<pb::Evidence> for Misbehavior {
        type Error = Error;

        fn try_from(evidence: pb::Evidence) -> Result<Self, Self::Error> {
            let kind = match evidence.r#type {
                0 => MisbehaviorKind::Unknown,
                1 => MisbehaviorKind::DuplicateVote,
                2 => MisbehaviorKind::LightClientAttack,
                _ => return Err(Error::invalid_evidence()),
            };

            Ok(Self {
                kind,
                validator: evidence
                    .validator
                    .ok_or_else(Error::missing_validator)?
                    .try_into()?,
                height: evidence.height.try_into()?,
                time: evidence
                    .time
                    .ok_or_else(Error::missing_timestamp)?
                    .try_into()?,
                total_voting_power: evidence.total_voting_power.try_into()?,
            })
        }
    }

    impl Protobuf<pb::Evidence> for Misbehavior {}

    impl From<CommitInfo> for pb::LastCommitInfo {
        fn from(lci: CommitInfo) -> Self {
            Self {
                round: lci.round.into(),
                votes: lci.votes.into_iter().map(Into::into).collect(),
            }
        }
    }

    impl TryFrom<pb::LastCommitInfo> for CommitInfo {
        type Error = Error;

        fn try_from(lci: pb::LastCommitInfo) -> Result<Self, Self::Error> {
            Ok(Self {
                round: lci.round.try_into()?,
                votes: lci
                    .votes
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
            })
        }
    }

    impl Protobuf<pb::LastCommitInfo> for CommitInfo {}

    impl From<Snapshot> for pb::Snapshot {
        fn from(snapshot: Snapshot) -> Self {
            Self {
                height: snapshot.height.into(),
                format: snapshot.format,
                chunks: snapshot.chunks,
                hash: snapshot.hash,
                metadata: snapshot.metadata,
            }
        }
    }

    impl TryFrom<pb::Snapshot> for Snapshot {
        type Error = Error;

        fn try_from(snapshot: pb::Snapshot) -> Result<Self, Self::Error> {
            Ok(Self {
                height: snapshot.height.try_into()?,
                format: snapshot.format,
                chunks: snapshot.chunks,
                hash: snapshot.hash,
                metadata: snapshot.metadata,
            })
        }
    }

    impl Protobuf<pb::Snapshot> for Snapshot {}
}

mod v0_37 {
    use super::{CommitInfo, Misbehavior, MisbehaviorKind, Snapshot, Validator, VoteInfo};
    use crate::{prelude::*, Error};
    use tendermint_proto::v0_37::abci as pb;
    use tendermint_proto::Protobuf;

    use bytes::Bytes;

    impl From<Validator> for pb::Validator {
        fn from(v: Validator) -> Self {
            Self {
                address: Bytes::copy_from_slice(&v.address[..]),
                power: v.power.into(),
            }
        }
    }

    impl TryFrom<pb::Validator> for Validator {
        type Error = Error;

        fn try_from(vu: pb::Validator) -> Result<Self, Self::Error> {
            let address = if vu.address.len() == 20 {
                let mut bytes = [0u8; 20];
                bytes.copy_from_slice(&vu.address);
                bytes
            } else {
                return Err(Error::invalid_account_id_length());
            };

            Ok(Self {
                address,
                power: vu.power.try_into()?,
            })
        }
    }

    impl Protobuf<pb::Validator> for Validator {}

    impl From<VoteInfo> for pb::VoteInfo {
        fn from(vi: VoteInfo) -> Self {
            Self {
                validator: Some(vi.validator.into()),
                signed_last_block: vi.signed_last_block,
            }
        }
    }

    impl TryFrom<pb::VoteInfo> for VoteInfo {
        type Error = Error;

        fn try_from(vi: pb::VoteInfo) -> Result<Self, Self::Error> {
            Ok(Self {
                validator: vi
                    .validator
                    .ok_or_else(Error::missing_validator)?
                    .try_into()?,
                signed_last_block: vi.signed_last_block,
            })
        }
    }

    impl Protobuf<pb::VoteInfo> for VoteInfo {}

    // ExtendedVoteInfo is defined in 0.37, but the vote_extension field is always nil,
    // so we can omit it from VoteInfo for the time being.

    impl From<VoteInfo> for pb::ExtendedVoteInfo {
        fn from(vi: VoteInfo) -> Self {
            Self {
                validator: Some(vi.validator.into()),
                signed_last_block: vi.signed_last_block,
                vote_extension: Default::default(),
            }
        }
    }

    impl TryFrom<pb::ExtendedVoteInfo> for VoteInfo {
        type Error = Error;

        fn try_from(vi: pb::ExtendedVoteInfo) -> Result<Self, Self::Error> {
            Ok(Self {
                validator: vi
                    .validator
                    .ok_or_else(Error::missing_validator)?
                    .try_into()?,
                signed_last_block: vi.signed_last_block,
            })
        }
    }

    impl Protobuf<pb::ExtendedVoteInfo> for VoteInfo {}

    impl From<Misbehavior> for pb::Misbehavior {
        fn from(evidence: Misbehavior) -> Self {
            Self {
                r#type: evidence.kind as i32,
                validator: Some(evidence.validator.into()),
                height: evidence.height.into(),
                time: Some(evidence.time.into()),
                total_voting_power: evidence.total_voting_power.into(),
            }
        }
    }

    impl TryFrom<pb::Misbehavior> for Misbehavior {
        type Error = Error;

        fn try_from(evidence: pb::Misbehavior) -> Result<Self, Self::Error> {
            let kind = match evidence.r#type {
                0 => MisbehaviorKind::Unknown,
                1 => MisbehaviorKind::DuplicateVote,
                2 => MisbehaviorKind::LightClientAttack,
                _ => return Err(Error::invalid_evidence()),
            };

            Ok(Self {
                kind,
                validator: evidence
                    .validator
                    .ok_or_else(Error::missing_validator)?
                    .try_into()?,
                height: evidence.height.try_into()?,
                time: evidence
                    .time
                    .ok_or_else(Error::missing_timestamp)?
                    .try_into()?,
                total_voting_power: evidence.total_voting_power.try_into()?,
            })
        }
    }

    impl Protobuf<pb::Misbehavior> for Misbehavior {}

    // The CommitInfo domain type represents both CommitInfo and ExtendedCommitInfo
    // as defined in protobuf for 0.37.

    impl From<CommitInfo> for pb::CommitInfo {
        fn from(lci: CommitInfo) -> Self {
            Self {
                round: lci.round.into(),
                votes: lci.votes.into_iter().map(Into::into).collect(),
            }
        }
    }

    impl TryFrom<pb::CommitInfo> for CommitInfo {
        type Error = Error;

        fn try_from(lci: pb::CommitInfo) -> Result<Self, Self::Error> {
            Ok(Self {
                round: lci.round.try_into()?,
                votes: lci
                    .votes
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
            })
        }
    }

    impl Protobuf<pb::CommitInfo> for CommitInfo {}

    impl From<CommitInfo> for pb::ExtendedCommitInfo {
        fn from(lci: CommitInfo) -> Self {
            Self {
                round: lci.round.into(),
                votes: lci.votes.into_iter().map(Into::into).collect(),
            }
        }
    }

    impl TryFrom<pb::ExtendedCommitInfo> for CommitInfo {
        type Error = Error;

        fn try_from(lci: pb::ExtendedCommitInfo) -> Result<Self, Self::Error> {
            Ok(Self {
                round: lci.round.try_into()?,
                votes: lci
                    .votes
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?,
            })
        }
    }

    impl Protobuf<pb::ExtendedCommitInfo> for CommitInfo {}

    impl From<Snapshot> for pb::Snapshot {
        fn from(snapshot: Snapshot) -> Self {
            Self {
                height: snapshot.height.into(),
                format: snapshot.format,
                chunks: snapshot.chunks,
                hash: snapshot.hash,
                metadata: snapshot.metadata,
            }
        }
    }

    impl TryFrom<pb::Snapshot> for Snapshot {
        type Error = Error;

        fn try_from(snapshot: pb::Snapshot) -> Result<Self, Self::Error> {
            Ok(Self {
                height: snapshot.height.try_into()?,
                format: snapshot.format,
                chunks: snapshot.chunks,
                hash: snapshot.hash,
                metadata: snapshot.metadata,
            })
        }
    }

    impl Protobuf<pb::Snapshot> for Snapshot {}
}
