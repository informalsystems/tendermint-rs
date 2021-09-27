//! Commits to a Tendermint blockchain

use crate::block::commit_sig::CommitSig;
use crate::block::{Height, Id, Round};
use crate::error::Error;
use crate::prelude::*;
use core::convert::{TryFrom, TryInto};
use serde::{Deserialize, Serialize, Serializer};
use tendermint_proto::types::{Commit as RawCommit, CommitSig as RawCommitSig};

/// Commit contains the justification (ie. a set of signatures) that a block was committed by a set
/// of validators.
/// TODO: Update links below!
/// <https://github.com/tendermint/tendermint/blob/51dc810d041eaac78320adc6d53ad8b160b06601/types/block.go#L486-L502>
/// <https://github.com/tendermint/spec/blob/d46cd7f573a2c6a2399fcab2cde981330aa63f37/spec/core/data_structures.md#lastcommit>
#[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
#[serde(try_from = "RawCommit")] // Used by testgen Generator trait
pub struct Commit {
    /// Block height
    pub height: Height,

    /// Round
    pub round: Round,

    /// Block ID
    pub block_id: Id,

    /// Signatures
    pub signatures: Vec<CommitSig>,
}

impl TryFrom<RawCommit> for Commit {
    type Error = Error;

    fn try_from(value: RawCommit) -> Result<Self, Self::Error> {
        let signatures: Result<Vec<CommitSig>, Error> = value
            .signatures
            .into_iter()
            .map(TryFrom::try_from)
            .collect();
        Ok(Self {
            height: value.height.try_into()?,
            round: value.round.try_into()?,
            block_id: value
                .block_id
                .ok_or_else(|| Error::invalid_block("missing block id".to_string()))?
                .try_into()?, /* gogoproto.nullable = false */
            signatures: signatures?,
        })
    }
}

impl TryFrom<Commit> for RawCommit {
    type Error = Error;

    fn try_from(value: Commit) -> Result<Self, Error> {
        let signatures: Vec<RawCommitSig> = value
            .signatures
            .into_iter()
            .map(CommitSig::try_into)
            .collect::<Result<_, Error>>()?;

        Ok(RawCommit {
            height: value.height.into(),
            round: value.round.into(),
            block_id: Some(value.block_id.into()),
            signatures,
        })
    }
}

impl Default for Commit {
    fn default() -> Self {
        Commit {
            // The default Height is 1, but the default commit is an empty commit with height = 0.
            height: Height::from(0_u32),
            round: Default::default(),
            block_id: Default::default(),
            signatures: vec![],
        }
    }
}

impl Serialize for Commit {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let raw: RawCommit = self.clone().try_into().map_err(serde::ser::Error::custom)?;

        raw.serialize(serializer)
    }
}
