use crate::{block, chain, timestamp::Timestamp};

/// Information about a particular Tendermint blockchain
#[derive(Clone, Debug)]
pub struct Info {
    /// Chain identifier (e.g. 'gaia-9000')
    pub id: chain::Id,

    /// Current block height of the chain
    pub height: block::Height,

    /// Last block ID seen for this chain
    pub last_block_id: Option<block::Id>,

    /// Current consensus time (if available)
    pub time: Option<Timestamp>,
}

impl Info {
    /// Create information about a particular network
    pub fn new<I>(id: I) -> Self
    where
        I: Into<chain::Id>,
    {
        Self {
            id: id.into(),
            height: Default::default(),
            last_block_id: None,
            time: None,
        }
    }
}
