//! State of a particular Tendermint network (a.k.a. chain)

use crate::block;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Tendermint consensus state
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConsensusState {
    /// Current block height
    pub height: block::Height,

    /// Current consensus round
    pub round: i64,

    /// Current consensus step
    pub step: i8,

    /// Block ID being proposed (if available)
    pub block_id: Option<block::Id>,
}
