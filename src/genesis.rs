//! Genesis data

use crate::{chain, consensus, Hash, Time};
use serde::{Deserialize, Serialize};

/// Genesis data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Genesis<AppState = serde_json::Value> {
    /// Time of genesis
    pub genesis_time: Time,

    /// Chain ID
    pub chain_id: chain::Id,

    /// Consensus parameters
    pub consensus_params: consensus::Params,

    /// App hash
    pub app_hash: Hash,

    /// App state
    pub app_state: AppState,
}
