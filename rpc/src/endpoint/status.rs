//! `/status` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};

use tendermint::{block, node, serializers, validator, Hash, Time};

/// Node status request
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request;

impl crate::Request for Request {
    type Response = Response;

    fn method(&self) -> crate::Method {
        crate::Method::Status
    }
}

/// Status responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Node information
    pub node_info: node::Info,

    /// Sync information
    pub sync_info: SyncInfo,

    /// Validator information
    pub validator_info: validator::Info,
}

impl crate::Response for Response {}

/// Sync information
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncInfo {
    /// Latest block hash
    #[serde(deserialize_with = "serializers::parse_non_empty_hash")]
    pub latest_block_hash: Option<Hash>,

    /// Latest app hash
    #[serde(deserialize_with = "serializers::parse_non_empty_hash")]
    pub latest_app_hash: Option<Hash>,

    /// Latest block height
    pub latest_block_height: block::Height,

    /// Latest block time
    pub latest_block_time: Time,

    /// Are we catching up?
    pub catching_up: bool,
}
