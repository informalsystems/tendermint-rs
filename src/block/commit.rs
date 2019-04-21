//! Commits to a Tendermint blockchain

use crate::{block, Vote};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Last commit to a particular blockchain: +2/3 precommit signatures.
///
/// <https://github.com/tendermint/tendermint/blob/master/docs/spec/blockchain/blockchain.md#lastcommit>
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct LastCommit {
    /// Block ID of the last commit
    pub block_id: block::Id,

    /// Precommits
    pub precommits: Vec<Option<Vote>>,
}
