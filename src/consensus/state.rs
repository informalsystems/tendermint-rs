//! Tendermint consensus state

pub use crate::block;
pub use std::cmp::Ordering;
#[cfg(feature = "serde")]
use {
    crate::serializers,
    serde::{Deserialize, Serialize},
};

/// Tendermint consensus state
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct State {
    /// Current block height
    pub height: block::Height,

    /// Current consensus round
    #[cfg_attr(
        feature = "serde",
        serde(
            serialize_with = "serializers::serialize_i64",
            deserialize_with = "serializers::parse_i64"
        )
    )]
    pub round: i64,

    /// Current consensus step
    pub step: i8,

    /// Block ID being proposed (if available)
    pub block_id: Option<block::Id>,
}

impl Ord for State {
    fn cmp(&self, other: &State) -> Ordering {
        if self.height < other.height {
            Ordering::Less
        } else if self.height == other.height {
            if self.round < other.round {
                Ordering::Less
            } else if self.round == other.round {
                self.step.cmp(&other.step)
            } else {
                Ordering::Greater
            }
        } else {
            Ordering::Greater
        }
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &State) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::State;
    use crate::block;

    #[test]
    fn state_ord_test() {
        let new = State {
            height: block::Height::from(9001u64),
            round: 0,
            step: 0,
            block_id: None,
        };

        let old = State {
            height: block::Height::from(1001u64),
            round: 1,
            step: 0,
            block_id: None,
        };

        let older = State {
            height: block::Height::from(1001u64),
            round: 0,
            step: 0,
            block_id: None,
        };

        let oldest = State {
            height: block::Height::default(),
            round: 0,
            step: 0,
            block_id: None,
        };

        assert!(old < new);
        assert!(older < old);
        assert!(oldest < older);
        assert!(oldest < new);
    }
}
