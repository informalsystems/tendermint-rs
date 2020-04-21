//! Genesis data

use crate::{chain, consensus, validator, Time};
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

    /// Validators
    pub validators: Vec<validator::Info>,

    /// App hash
    #[serde(skip_serializing_if = "Vec::is_empty", with = "serde_bytes")]
    pub app_hash: Vec<u8>,

    /// App state
    #[serde(default)]
    pub app_state: AppState,
}
