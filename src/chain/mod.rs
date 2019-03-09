//! Information about particular Tendermint blockchain networks

mod guard;
pub mod key;
mod registry;

pub use self::{guard::Guard, registry::REGISTRY};
use crate::config::chain::ChainConfig;
pub use tendermint::chain::Id;

/// Information about a particular Tendermint blockchain network
pub struct Chain {
    /// ID of a particular chain
    pub id: Id,

    /// Key format configuration
    pub key_format: key::Format,
}

impl<'a> From<&ChainConfig> for Chain {
    fn from(config: &ChainConfig) -> Chain {
        Self {
            id: config.id,
            key_format: config.key_format.clone(),
        }
    }
}
