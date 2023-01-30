use serde::{Deserialize, Serialize};

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
