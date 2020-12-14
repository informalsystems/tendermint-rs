//! `/status` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};

use tendermint::{block, node, validator, AppHash, Hash, Time};

/// Node status request
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request;

impl crate::Request for Request {
    type Response = Response;

    fn method(&self) -> crate::Method {
        crate::Method::Status
    }
}

impl crate::SimpleRequest for Request {}

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
    #[serde(with = "tendermint::serializers::hash")]
    pub latest_block_hash: Hash,

    /// Latest app hash
    #[serde(with = "tendermint::serializers::apphash")]
    pub latest_app_hash: AppHash,

    /// Latest block height
    pub latest_block_height: block::Height,

    /// Latest block time
    pub latest_block_time: Time,

    /// Are we catching up?
    pub catching_up: bool,
}
