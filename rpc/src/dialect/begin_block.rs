use serde::{Deserialize, Serialize};

use tendermint::abci;

use crate::prelude::*;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct BeginBlock<Ev> {
    /// Events that occurred while beginning the block.
    #[serde(default = "Default::default")]
    pub events: Vec<Ev>,
}

impl<Ev> Default for BeginBlock<Ev> {
    fn default() -> Self {
        Self {
            events: Default::default(),
        }
    }
}

impl<Ev> From<BeginBlock<Ev>> for abci::response::BeginBlock
where
    Ev: Into<abci::Event>,
{
    fn from(msg: BeginBlock<Ev>) -> Self {
        Self {
            events: msg.events.into_iter().map(Into::into).collect(),
        }
    }
}

impl<Ev> From<abci::response::BeginBlock> for BeginBlock<Ev>
where
    abci::Event: Into<Ev>,
{
    fn from(value: abci::response::BeginBlock) -> Self {
        Self {
            events: value.events.into_iter().map(Into::into).collect(),
        }
    }
}
