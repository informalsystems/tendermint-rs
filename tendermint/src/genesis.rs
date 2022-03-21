//! Genesis data

use crate::prelude::*;
use crate::serializers;
use crate::{chain, consensus, validator, Time};
use serde::{Deserialize, Serialize};

/// Genesis data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Genesis<AppState = serde_json::Value> {
    /// Time of genesis
    pub genesis_time: Time,

    /// Chain ID
    pub chain_id: chain::Id,

    /// Starting height of the blockchain
    #[serde(with = "serializers::from_str")]
    pub initial_height: i64,

    /// Consensus parameters
    pub consensus_params: consensus::Params,

    /// Validators
    #[serde(default)]
    pub validators: Vec<validator::Info>,

    /// App hash
    #[serde(with = "serializers::bytes::hexstring")]
    pub app_hash: Vec<u8>,

    /// App state
    pub app_state: AppState,
}
