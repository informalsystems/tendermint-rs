//! Commits to a Tendermint blockchain

use crate::{block, Vote};
use serde::{Deserialize, Serialize};
use std::{ops::Deref, slice};

/// Commit contains the justification (ie. a set of signatures) that a block was committed by a set of validators.
///
/// <https://github.com/tendermint/tendermint/blob/51dc810d041eaac78320adc6d53ad8b160b06601/types/block.go#L486-L502>
/// <https://github.com/tendermint/tendermint/blob/master/docs/spec/blockchain/blockchain.md#lastcommit>
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Commit {
    /// Block ID of the last commit
    pub block_id: block::Id,

    /// Precommits
    pub precommits: Precommits,
}

/// Precommits which certify that a block is valid
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Precommits(Option<Vec<Option<Vote>>>);

impl Precommits {
    /// Create a new precommit collection
    pub fn new<I>(into_precommits: I) -> Self
    where
        I: Into<Vec<Option<Vote>>>,
    {
        Self(Some(into_precommits.into()))
    }

    /// Convert this collection of precommits into a vector
    pub fn into_vec(self) -> Vec<Option<Vote>> {
        self.iter().cloned().collect()
    }

    /// Iterate over the precommits in the collection
    pub fn iter(&self) -> slice::Iter<'_, Option<Vote>> {
        self.as_ref().iter()
    }
}

impl AsRef<[Option<Vote>]> for Precommits {
    fn as_ref(&self) -> &[Option<Vote>] {
        self.0.as_ref().map(Vec::as_slice).unwrap_or_else(|| &[])
    }
}

impl Deref for Precommits {
    type Target = [Option<Vote>];

    fn deref(&self) -> &[Option<Vote>] {
        self.as_ref()
    }
}
