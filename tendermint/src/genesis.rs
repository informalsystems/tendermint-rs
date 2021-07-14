//! Genesis data

use crate::serializers;
use crate::{chain, consensus, validator, Time};
// use chrono::DateTime;
// use serde::de::Error;
use serde::{Deserialize, Serialize};
use sp_std::vec::Vec;

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
    #[serde(default)]
    pub app_state: AppState,
}
