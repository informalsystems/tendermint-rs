//! Chain configuration

mod hook;

pub use self::hook::HookConfig;
use crate::{chain, keyring};
use std::path::PathBuf;

/// Chain configuration
#[derive(Clone, Deserialize, Debug)]
pub struct ChainConfig {
    /// Chain ID of this Tendermint network/chain
    pub id: chain::Id,

    /// Key serialization format configuration for this chain
    pub key_format: keyring::Format,

    /// Path to chain-specific `priv_validator_state.json` file
    pub state_file: Option<PathBuf>,

    /// User-specified command to run to obtain the current block height for
    /// this chain. This will be executed at launch time to populate the
    /// initial block height if configured
    pub state_hook: Option<HookConfig>,
}
