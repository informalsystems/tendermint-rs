//! State of a particular Tendermint network (a.k.a. chain)

use crate::block;

/// Tendermint consensus state
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
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
