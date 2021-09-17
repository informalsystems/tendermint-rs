//! Tendermint consensus state

pub use crate::block;
use crate::prelude::*;
pub use core::{cmp::Ordering, fmt};
use serde::{Deserialize, Serialize};

/// Placeholder string to show when block ID is absent. Syntax from:
/// <https://tendermint.com/docs/spec/consensus/consensus.html>
pub const NIL_PLACEHOLDER: &str = "<nil>";

/// Tendermint consensus state
// Serde serialization for KMS state file read/write.
// https://github.com/informalsystems/tendermint-rs/issues/675
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct State {
    /// Current block height
    pub height: block::Height,

    /// Current consensus round
    pub round: block::Round,

    /// Current consensus step
    pub step: i8,

    /// Block ID being proposed (if available)
    #[serde(with = "tendermint_proto::serializers::optional")]
    pub block_id: Option<block::Id>,
}

impl State {
    /// Get short prefix of the block ID for debugging purposes (ala git)
    pub fn block_id_prefix(&self) -> String {
        self.block_id
            .as_ref()
            .map(block::Id::prefix)
            .unwrap_or_else(|| NIL_PLACEHOLDER.to_owned())
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}/{}", self.height, self.round, self.step)
    }
}

impl Ord for State {
    fn cmp(&self, other: &State) -> Ordering {
        match self.height.cmp(&other.height) {
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less,
            Ordering::Equal => match self.round.cmp(&other.round) {
                Ordering::Greater => Ordering::Greater,
                Ordering::Less => Ordering::Less,
                Ordering::Equal => self.step.cmp(&other.step),
            },
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
    use crate::Hash;
    use core::str::FromStr;

    #[test]
    fn state_ord_test() {
        let new = State {
            height: block::Height::from(9001_u32),
            round: block::Round::default(),
            step: 0,
            block_id: None,
        };

        let old = State {
            height: block::Height::from(1001_u32),
            round: block::Round::from(1_u16),
            step: 0,
            block_id: None,
        };

        let older = State {
            height: block::Height::from(1001_u32),
            round: block::Round::default(),
            step: 0,
            block_id: None,
        };

        let oldest = State {
            height: block::Height::default(),
            round: block::Round::default(),
            step: 0,
            block_id: None,
        };

        assert!(old < new);
        assert!(older < old);
        assert!(oldest < older);
        assert!(oldest < new);
    }

    #[test]
    fn state_deser_update_null_test() {
        // Testing that block_id == null is correctly deserialized.
        let state_json_string = r#"{
            "height": "5",
            "round": "1",
            "step": 6,
            "block_id": null
        }"#;
        let state: State = State {
            height: block::Height::from(5_u32),
            round: block::Round::from(1_u16),
            step: 6,
            block_id: None,
        };
        let state_from_json: State = serde_json::from_str(state_json_string).unwrap();
        assert_eq!(state_from_json, state);
    }

    #[test]
    fn state_deser_update_total_test() {
        // Testing, if total is correctly deserialized from string.
        // Note that we use 'parts' to test backwards compatibility.
        let state_json_string = r#"{
            "height": "5",
            "round": "1",
            "step": 6,
            "block_id": {
              "hash": "1234567890123456789012345678901234567890123456789012345678901234",
              "parts": {
                  "total": "1",
                  "hash": "1234567890123456789012345678901234567890123456789012345678901234"
              }
            }
        }"#;
        let state: State = State {
            height: block::Height::from(5_u32),
            round: block::Round::from(1_u16),
            step: 6,
            block_id: Some(block::Id {
                hash: Hash::from_str(
                    "1234567890123456789012345678901234567890123456789012345678901234",
                )
                .unwrap(),
                part_set_header: block::parts::Header::new(
                    1,
                    Hash::from_str(
                        "1234567890123456789012345678901234567890123456789012345678901234",
                    )
                    .unwrap(),
                )
                .unwrap(),
            }),
        };
        let state_from_json: State = serde_json::from_str(state_json_string).unwrap();
        assert_eq!(state_from_json, state);
    }
}
