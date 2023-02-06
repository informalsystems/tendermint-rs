use serde::{Deserialize, Serialize};

use tendermint::{abci, consensus, validator};

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

impl<Ev> From<EndBlock<Ev>> for abci::response::EndBlock
where
    Ev: Into<abci::Event>,
{
    fn from(msg: EndBlock<Ev>) -> Self {
        Self {
            events: msg.events.into_iter().map(Into::into).collect(),
            validator_updates: msg.validator_updates,
            consensus_param_updates: msg.consensus_param_updates,
        }
    }
}

impl<Ev> From<abci::response::EndBlock> for EndBlock<Ev>
where
    abci::Event: Into<Ev>,
{
    fn from(value: abci::response::EndBlock) -> Self {
        Self {
            events: value.events.into_iter().map(Into::into).collect(),
            validator_updates: value.validator_updates,
            consensus_param_updates: value.consensus_param_updates,
        }
    }
}
