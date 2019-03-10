//! Information about particular Tendermint blockchain networks

mod guard;
pub mod key;
mod registry;
pub mod state;

pub use self::{guard::Guard, registry::REGISTRY, state::LastSignState};
use crate::{config::chain::ChainConfig, error::KmsError};
use std::{path::PathBuf, sync::Mutex};
pub use tendermint::chain::Id;

/// Information about a particular Tendermint blockchain network
pub struct Chain {
    /// ID of a particular chain
    pub id: Id,

    /// Key format configuration
    pub key_format: key::Format,

    /// State from the last block signed for this chain
    pub state: Mutex<LastSignState>,
}

impl Chain {
    /// Attempt to create a `Chain` state from the given configuration
    pub fn from_config(config: &ChainConfig) -> Result<Chain, KmsError> {
        let state_file = match config.state_file {
            Some(ref path) => path.to_owned(),
            None => PathBuf::from(&format!("{}_priv_validator_state.json", config.id)),
        };

        let last_sign_state = LastSignState::load_state(state_file)?;

        Ok(Self {
            id: config.id,
            key_format: config.key_format.clone(),
            state: Mutex::new(last_sign_state),
        })
    }
}
