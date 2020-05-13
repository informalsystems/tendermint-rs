//! Commits to a Tendermint blockchain

use crate::block::commit_sig::CommitSig;
use crate::block::{Height, Id};
use crate::serializers;
use serde::{Deserialize, Serialize};
use std::{ops::Deref, slice};

/// Commit contains the justification (ie. a set of signatures) that a block was committed by a set
/// of validators.
/// TODO: Update links below!
/// <https://github.com/tendermint/tendermint/blob/51dc810d041eaac78320adc6d53ad8b160b06601/types/block.go#L486-L502>
/// <https://github.com/tendermint/tendermint/blob/master/docs/spec/blockchain/blockchain.md#lastcommit>
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Commit {
    /// Block height
    pub height: Height,

    /// Round
    #[serde(with = "serializers::from_str")]
    pub round: u64,

    /// Block ID
    pub block_id: Id,

    /// Signatures
    pub signatures: CommitSigs,
}

/// CommitSigs which certify that a block is valid
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CommitSigs(Vec<CommitSig>);

impl CommitSigs {
    /// Create a new CommitSig collection
    pub fn new<I>(into_commit_sigs: I) -> Self
    where
        I: Into<Vec<CommitSig>>,
    {
        Self(into_commit_sigs.into())
    }

    /// Convert this collection of CommitSigs into a vector
    pub fn into_vec(self) -> Vec<CommitSig> {
        self.0
    }

    /// Iterate over the CommitSigs in the collection
    pub fn iter(&self) -> slice::Iter<'_, CommitSig> {
        self.0.iter()
    }
}

impl AsRef<[CommitSig]> for CommitSigs {
    fn as_ref(&self) -> &[CommitSig] {
        self.0.as_slice()
    }
}

impl Deref for CommitSigs {
    type Target = [CommitSig];

    fn deref(&self) -> &[CommitSig] {
        self.as_ref()
    }
}

impl PartialEq for CommitSigs {
    fn eq(&self, other: &Self) -> bool {
        // Note: this is used for asserts in tests:
        self.0.clone().into_iter().eq(other.0.clone().into_iter())
    }
}
