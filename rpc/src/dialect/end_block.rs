use serde::{Deserialize, Serialize};

use tendermint::{consensus, validator};

use crate::prelude::*;
use crate::serializers;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct EndBlock<Ev> {
    /// Changes to the validator set, if any.
    ///
    /// Setting the voting power to 0 removes a validator.
    #[serde(with = "serializers::nullable")]
    pub validator_updates: Vec<validator::Update>,
    /// Changes to consensus parameters (optional).
    pub consensus_param_updates: Option<consensus::Params>,
    /// Events that occurred while ending the block.
    #[serde(default = "Default::default")]
    pub events: Vec<Ev>,
}

impl<Ev> Default for EndBlock<Ev> {
    fn default() -> Self {
        Self {
            validator_updates: Default::default(),
            consensus_param_updates: Default::default(),
            events: Default::default(),
        }
    }
}
