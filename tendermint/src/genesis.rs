//! Genesis data

use crate::serializers;
use crate::{chain, consensus, validator, Time};
use chrono::DateTime;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};
use std::convert::TryFrom;
use tendermint_proto::google::protobuf::Timestamp;

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
    pub app_hash: String,

    /// App state
    #[serde(default)]
    pub app_state: AppState,
}

/// Deserialize string into Time through Timestamp
pub fn deserialize_time<'de, D>(deserializer: D) -> Result<Time, D::Error>
where
    D: Deserializer<'de>,
{
    let value_string = String::deserialize(deserializer)?;
    let value_datetime = DateTime::parse_from_rfc3339(value_string.as_str())
        .map_err(|e| D::Error::custom(format!("{}", e)))?;
    Time::try_from(Timestamp {
        seconds: value_datetime.timestamp(),
        nanos: value_datetime.timestamp_subsec_nanos() as i32,
    })
    .map_err(|e| D::Error::custom(format!("{}", e)))
}
