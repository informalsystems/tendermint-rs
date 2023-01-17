//! Evidence of malfeasance by validators (i.e. signing conflicting votes).

use core::{
    convert::{TryFrom, TryInto},
    slice,
};

use serde::{Deserialize, Serialize};
use tendermint_proto::{
    google::protobuf::Duration as RawDuration,
    types::{
        evidence::Sum as RawSum, DuplicateVoteEvidence as RawDuplicateVoteEvidence,
        Evidence as RawEvidence, EvidenceList as RawEvidenceList,
        EvidenceParams as RawEvidenceParams, LightBlock as RawLightBlock,
        LightClientAttackEvidence as RawLightClientAttackEvidence,
    },
    Protobuf,
};

use crate::{
    block::signed_header::SignedHeader, block::Height, error::Error, prelude::*, serializers,
    validator, vote::Power, Time, Vote,
};

/// Evidence of malfeasance by validators (i.e. signing conflicting votes).
///
/// <https://github.com/tendermint/spec/blob/d46cd7f573a2c6a2399fcab2cde981330aa63f37/spec/core/data_structures.md#evidence>
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawEvidence", into = "RawEvidence")] // Used by RPC /broadcast_evidence endpoint
pub enum Evidence {
    /// Duplicate vote evidence
    DuplicateVote(Box<DuplicateVoteEvidence>),

    /// LightClient attack evidence
    LightClientAttack(Box<LightClientAttackEvidence>),
}

impl From<LightClientAttackEvidence> for Evidence {
    fn from(ev: LightClientAttackEvidence) -> Self {
        Self::LightClientAttack(Box::new(ev))
    }
}

impl From<DuplicateVoteEvidence> for Evidence {
    fn from(ev: DuplicateVoteEvidence) -> Self {
        Self::DuplicateVote(Box::new(ev))
    }
}

impl Protobuf<RawEvidence> for Evidence {}

impl TryFrom<RawEvidence> for Evidence {
    type Error = Error;

    fn try_from(value: RawEvidence) -> Result<Self, Self::Error> {
        match value.sum.ok_or_else(Error::invalid_evidence)? {
            RawSum::DuplicateVoteEvidence(ev) => {
                Ok(Evidence::DuplicateVote(Box::new(ev.try_into()?)))
            },
            RawSum::LightClientAttackEvidence(ev) => {
                Ok(Evidence::LightClientAttack(Box::new(ev.try_into()?)))
            },
        }
    }
}

impl From<Evidence> for RawEvidence {
    fn from(value: Evidence) -> Self {
        match value {
            Evidence::DuplicateVote(ev) => RawEvidence {
                sum: Some(RawSum::DuplicateVoteEvidence((*ev).into())),
            },
            Evidence::LightClientAttack(ev) => RawEvidence {
                sum: Some(RawSum::LightClientAttackEvidence((*ev).into())),
            },
        }
    }
}

/// Duplicate vote evidence
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DuplicateVoteEvidence {
    pub vote_a: Vote,
    pub vote_b: Vote,
    pub total_voting_power: Power,
    pub validator_power: Power,
    pub timestamp: Time,
}

impl Protobuf<RawDuplicateVoteEvidence> for DuplicateVoteEvidence {}

impl TryFrom<RawDuplicateVoteEvidence> for DuplicateVoteEvidence {
    type Error = Error;

    fn try_from(value: RawDuplicateVoteEvidence) -> Result<Self, Self::Error> {
        Ok(Self {
            vote_a: value
                .vote_a
                .ok_or_else(Error::missing_evidence)?
                .try_into()?,
            vote_b: value
                .vote_b
                .ok_or_else(Error::missing_evidence)?
                .try_into()?,
            total_voting_power: value.total_voting_power.try_into()?,
            validator_power: value.validator_power.try_into()?,
            timestamp: value
                .timestamp
                .ok_or_else(Error::missing_timestamp)?
                .try_into()?,
        })
    }
}

impl From<DuplicateVoteEvidence> for RawDuplicateVoteEvidence {
    fn from(value: DuplicateVoteEvidence) -> Self {
        RawDuplicateVoteEvidence {
            vote_a: Some(value.vote_a.into()),
            vote_b: Some(value.vote_b.into()),
            total_voting_power: value.total_voting_power.into(),
            validator_power: value.total_voting_power.into(),
            timestamp: Some(value.timestamp.into()),
        }
    }
}

impl DuplicateVoteEvidence {
    pub fn new(vote_a: Vote, vote_b: Vote) -> Result<Self, Error> {
        if vote_a.height != vote_b.height {
            return Err(Error::invalid_evidence());
        }

        // TODO: make more assumptions about what is considered a valid evidence for duplicate vote
        Ok(Self {
            vote_a,
            vote_b,
            total_voting_power: Default::default(),
            validator_power: Default::default(),
            timestamp: Time::unix_epoch(),
        })
    }

    pub fn votes(&self) -> (&Vote, &Vote) {
        (&self.vote_a, &self.vote_b)
    }
}

/// Conflicting block detected in light client attack
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConflictingBlock {
    pub signed_header: SignedHeader,
    pub validator_set: validator::Set,
}

impl Protobuf<RawLightBlock> for ConflictingBlock {}

impl TryFrom<RawLightBlock> for ConflictingBlock {
    type Error = Error;

    fn try_from(value: RawLightBlock) -> Result<Self, Self::Error> {
        Ok(ConflictingBlock {
            signed_header: value
                .signed_header
                .ok_or_else(Error::missing_evidence)?
                .try_into()?,
            validator_set: value
                .validator_set
                .ok_or_else(Error::missing_evidence)?
                .try_into()?,
        })
    }
}

impl From<ConflictingBlock> for RawLightBlock {
    fn from(value: ConflictingBlock) -> Self {
        RawLightBlock {
            signed_header: Some(value.signed_header.into()),
            validator_set: Some(value.validator_set.into()),
        }
    }
}

/// Light client attack evidence
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LightClientAttackEvidence {
    pub conflicting_block: ConflictingBlock,
    pub common_height: Height,
    pub byzantine_validators: Vec<validator::Info>,
    pub total_voting_power: Power,
    pub timestamp: Time,
}

impl Protobuf<RawLightClientAttackEvidence> for LightClientAttackEvidence {}

impl TryFrom<RawLightClientAttackEvidence> for LightClientAttackEvidence {
    type Error = Error;

    fn try_from(ev: RawLightClientAttackEvidence) -> Result<Self, Self::Error> {
        Ok(LightClientAttackEvidence {
            conflicting_block: ev
                .conflicting_block
                .ok_or_else(Error::missing_evidence)?
                .try_into()?,
            common_height: ev.common_height.try_into()?,
            byzantine_validators: ev
                .byzantine_validators
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            total_voting_power: ev.total_voting_power.try_into()?,
            timestamp: ev
                .timestamp
                .ok_or_else(Error::missing_timestamp)?
                .try_into()?,
        })
    }
}

impl From<LightClientAttackEvidence> for RawLightClientAttackEvidence {
    fn from(ev: LightClientAttackEvidence) -> Self {
        RawLightClientAttackEvidence {
            conflicting_block: Some(ev.conflicting_block.into()),
            common_height: ev.common_height.into(),
            byzantine_validators: ev
                .byzantine_validators
                .into_iter()
                .map(Into::into)
                .collect(),
            total_voting_power: ev.total_voting_power.into(),
            timestamp: Some(ev.timestamp.into()),
        }
    }
}

/// Evidence data is a wrapper for a list of `Evidence`.
///
/// <https://github.com/tendermint/spec/blob/d46cd7f573a2c6a2399fcab2cde981330aa63f37/spec/core/data_structures.md#evidencedata>
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawEvidenceList", into = "RawEvidenceList")]
pub struct Data {
    evidence: Option<Vec<Evidence>>,
}

impl Protobuf<RawEvidenceList> for Data {}

impl TryFrom<RawEvidenceList> for Data {
    type Error = Error;
    fn try_from(value: RawEvidenceList) -> Result<Self, Self::Error> {
        if value.evidence.is_empty() {
            return Ok(Self { evidence: None });
        }

        let evidence: Result<Vec<Evidence>, Error> =
            value.evidence.into_iter().map(TryInto::try_into).collect();

        Ok(Self {
            evidence: Some(evidence?),
        })
    }
}

impl From<Data> for RawEvidenceList {
    fn from(value: Data) -> Self {
        RawEvidenceList {
            evidence: value
                .evidence
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl Data {
    /// Create a new evidence data collection
    pub fn new<I>(into_evidence: I) -> Data
    where
        I: Into<Vec<Evidence>>,
    {
        Data {
            evidence: Some(into_evidence.into()),
        }
    }

    /// Convert this evidence data into a vector
    pub fn into_vec(self) -> Vec<Evidence> {
        self.iter().cloned().collect()
    }

    /// Iterate over the evidence data
    pub fn iter(&self) -> slice::Iter<'_, Evidence> {
        self.as_ref().iter()
    }
}

impl AsRef<[Evidence]> for Data {
    fn as_ref(&self) -> &[Evidence] {
        self.evidence.as_deref().unwrap_or(&[])
    }
}

/// EvidenceParams determine how we handle evidence of malfeasance.
///
/// [Tendermint documentation](https://docs.tendermint.com/master/spec/core/data_structures.html#evidenceparams)
#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
// TODO: This struct is ready to be converted through tendermint_proto::types::EvidenceParams.
// https://github.com/informalsystems/tendermint-rs/issues/741
pub struct Params {
    /// Max age of evidence, in blocks.
    #[serde(with = "serializers::from_str")]
    pub max_age_num_blocks: u64,

    /// Max age of evidence, in time.
    ///
    /// It should correspond with an app's "unbonding period" or other similar
    /// mechanism for handling [Nothing-At-Stake attacks][nas].
    ///
    /// [nas]: https://github.com/ethereum/wiki/wiki/Proof-of-Stake-FAQ#what-is-the-nothing-at-stake-problem-and-how-can-it-be-fixed
    pub max_age_duration: Duration,

    /// This sets the maximum size of total evidence in bytes that can be
    /// committed in a single block, and should fall comfortably under the max
    /// block bytes. The default is 1048576 or 1MB.
    #[serde(with = "serializers::from_str", default)]
    pub max_bytes: i64,
}

impl Protobuf<RawEvidenceParams> for Params {}

impl TryFrom<RawEvidenceParams> for Params {
    type Error = Error;

    fn try_from(value: RawEvidenceParams) -> Result<Self, Self::Error> {
        Ok(Self {
            max_age_num_blocks: value
                .max_age_num_blocks
                .try_into()
                .map_err(Error::negative_max_age_num)?,
            max_age_duration: value
                .max_age_duration
                .ok_or_else(Error::missing_max_age_duration)?
                .try_into()?,
            max_bytes: value.max_bytes,
        })
    }
}

impl From<Params> for RawEvidenceParams {
    fn from(value: Params) -> Self {
        Self {
            // TODO: Implement proper domain types so this becomes infallible
            max_age_num_blocks: value.max_age_num_blocks.try_into().unwrap(),
            max_age_duration: Some(value.max_age_duration.into()),
            max_bytes: value.max_bytes,
        }
    }
}

/// `Duration` is a wrapper around `core::time::Duration` essentially, to keep the usages look cleaner,
/// i.e. you can avoid using serde annotations everywhere
///
/// TODO: harmonize google::protobuf::Duration, core::time::Duration and this. Too many structs.
/// <https://github.com/informalsystems/tendermint-rs/issues/741>
#[derive(Copy, Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Duration(#[serde(with = "serializers::time_duration")] pub core::time::Duration);

impl From<Duration> for core::time::Duration {
    fn from(d: Duration) -> core::time::Duration {
        d.0
    }
}

impl Protobuf<RawDuration> for Duration {}

impl TryFrom<RawDuration> for Duration {
    type Error = Error;

    fn try_from(value: RawDuration) -> Result<Self, Self::Error> {
        Ok(Self(core::time::Duration::new(
            value.seconds.try_into().map_err(Error::integer_overflow)?,
            value.nanos.try_into().map_err(Error::integer_overflow)?,
        )))
    }
}

impl From<Duration> for RawDuration {
    fn from(value: Duration) -> Self {
        // TODO: make the struct into a proper domaintype so this becomes infallible.
        Self {
            seconds: value.0.as_secs() as i64,
            nanos: value.0.subsec_nanos() as i32,
        }
    }
}
