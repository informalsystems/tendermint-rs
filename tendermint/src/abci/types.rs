//! ABCI-specific data types used in requests and responses.
//!
//! These types have changes from the core data structures to better accomodate
//! ABCI applications.
//!
//! [ABCI documentation](https://docs.tendermint.com/master/spec/abci/abci.html#data-types)

use crate::prelude::*;

use core::convert::{TryFrom, TryInto};

use bytes::Bytes;
use chrono::{DateTime, Utc};

use crate::PublicKey;

/// A validator address with voting power.
///
/// [ABCI documentation](https://docs.tendermint.com/master/spec/abci/abci.html#validator)
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Validator {
    /// The validator's address (the first 20 bytes of `SHA256(public_key)`).
    pub address: [u8; 20],
    /// The voting power of the validator.
    pub power: i64,
}

/// A change to the validator set.
///
/// Used to inform Tendermint of changes to the validator set.
///
/// [ABCI documentation](https://docs.tendermint.com/master/spec/abci/abci.html#validatorupdate)
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ValidatorUpdate {
    /// The validator's public key.
    pub pub_key: PublicKey,
    /// The validator's voting power.
    pub power: i64,
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

/// The possible kinds of [`Evidence`].
///
/// Note: the
/// [ABCI documentation](https://docs.tendermint.com/master/spec/abci/abci.html#evidencetype-2)
/// calls this `EvidenceType`, but we follow the Rust convention and name it `EvidenceKind`
/// to avoid confusion with Rust types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum EvidenceKind {
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
pub struct Evidence {
    /// The kind of evidence.
    ///
    /// Note: this field is called `type` in the protobuf, but we call it `kind`
    /// to avoid the Rust keyword.
    pub kind: EvidenceKind,
    /// The offending validator.
    pub validator: Validator,
    /// The height when the offense occurred.
    pub height: i64,
    /// The corresponding time when the offense occurred.
    pub time: DateTime<Utc>,
    /// Total voting power of the validator set at `height`.
    ///
    /// This is included in case the ABCI application does not store historical
    /// validators, cf.
    /// [#4581](https://github.com/tendermint/tendermint/issues/4581)
    pub total_voting_power: i64,
}

/// Information on the last block commit.
///
/// [ABCI documentation](https://docs.tendermint.com/master/spec/abci/abci.html#lastcommitinfo)
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LastCommitInfo {
    /// The commit round.
    ///
    /// Reflects the total number of rounds it took to come to consensus for the
    /// current block.
    pub round: i32,
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
    pub height: u64,
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

// XXX(hdevalence): these all use &'static str for now, this should be fixed
// to align with the crate's error-handling strategy.

use tendermint_proto::abci as pb;
use tendermint_proto::Protobuf;

impl From<Validator> for pb::Validator {
    fn from(v: Validator) -> Self {
        Self {
            address: Bytes::copy_from_slice(&v.address[..]),
            power: v.power,
        }
    }
}

impl TryFrom<pb::Validator> for Validator {
    type Error = crate::Error;

    fn try_from(vu: pb::Validator) -> Result<Self, Self::Error> {
        let address = if vu.address.len() == 20 {
            let mut bytes = [0u8; 20];
            bytes.copy_from_slice(&vu.address);
            bytes
        } else {
            return Err("wrong address length".into());
        };

        Ok(Self {
            address,
            power: vu.power,
        })
    }
}

impl Protobuf<pb::Validator> for Validator {}

impl From<ValidatorUpdate> for pb::ValidatorUpdate {
    fn from(vu: ValidatorUpdate) -> Self {
        Self {
            pub_key: Some(vu.pub_key.into()),
            power: vu.power,
        }
    }
}

impl TryFrom<pb::ValidatorUpdate> for ValidatorUpdate {
    type Error = crate::Error;

    fn try_from(vu: pb::ValidatorUpdate) -> Result<Self, Self::Error> {
        Ok(Self {
            pub_key: vu.pub_key.ok_or("missing public key")?.try_into()?,
            power: vu.power,
        })
    }
}

impl Protobuf<pb::ValidatorUpdate> for ValidatorUpdate {}

impl From<VoteInfo> for pb::VoteInfo {
    fn from(vi: VoteInfo) -> Self {
        Self {
            validator: Some(vi.validator.into()),
            signed_last_block: vi.signed_last_block,
        }
    }
}

impl TryFrom<pb::VoteInfo> for VoteInfo {
    type Error = crate::Error;

    fn try_from(vi: pb::VoteInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            validator: vi.validator.ok_or("missing validator")?.try_into()?,
            signed_last_block: vi.signed_last_block,
        })
    }
}

impl Protobuf<pb::VoteInfo> for VoteInfo {}

impl From<Evidence> for pb::Evidence {
    fn from(evidence: Evidence) -> Self {
        Self {
            r#type: evidence.kind as i32,
            validator: Some(evidence.validator.into()),
            height: evidence.height,
            time: Some(evidence.time.into()),
            total_voting_power: evidence.total_voting_power,
        }
    }
}

impl TryFrom<pb::Evidence> for Evidence {
    type Error = crate::Error;

    fn try_from(evidence: pb::Evidence) -> Result<Self, Self::Error> {
        let kind = match evidence.r#type {
            0 => EvidenceKind::Unknown,
            1 => EvidenceKind::DuplicateVote,
            2 => EvidenceKind::LightClientAttack,
            _ => Err("unknown evidence kind")?,
        };

        Ok(Self {
            kind,
            validator: evidence.validator.ok_or("missing validator")?.try_into()?,
            height: evidence.height,
            time: evidence.time.ok_or("missing time")?.into(),
            total_voting_power: evidence.total_voting_power,
        })
    }
}

impl Protobuf<pb::Evidence> for Evidence {}

impl From<LastCommitInfo> for pb::LastCommitInfo {
    fn from(lci: LastCommitInfo) -> Self {
        Self {
            round: lci.round,
            votes: lci.votes.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<pb::LastCommitInfo> for LastCommitInfo {
    type Error = crate::Error;

    fn try_from(lci: pb::LastCommitInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            round: lci.round,
            votes: lci
                .votes
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl Protobuf<pb::LastCommitInfo> for LastCommitInfo {}

impl From<Snapshot> for pb::Snapshot {
    fn from(snapshot: Snapshot) -> Self {
        Self {
            height: snapshot.height,
            format: snapshot.format,
            chunks: snapshot.chunks,
            hash: snapshot.hash,
            metadata: snapshot.metadata,
        }
    }
}

impl TryFrom<pb::Snapshot> for Snapshot {
    type Error = crate::Error;

    fn try_from(snapshot: pb::Snapshot) -> Result<Self, Self::Error> {
        Ok(Self {
            height: snapshot.height,
            format: snapshot.format,
            chunks: snapshot.chunks,
            hash: snapshot.hash,
            metadata: snapshot.metadata,
        })
    }
}

impl Protobuf<pb::Snapshot> for Snapshot {}
