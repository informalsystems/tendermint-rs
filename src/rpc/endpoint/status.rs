//! `/status` endpoint JSONRPC wrapper

use crate::{block, node, rpc, validator, Hash, Timestamp};
use serde::{Deserialize, Serialize};

/// Node status request
#[derive(Debug, Default)]
pub struct Request;

impl rpc::Request for Request {
    type Response = Response;

    fn path(&self) -> rpc::request::Path {
        "/status".parse().unwrap()
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

impl rpc::Response for Response {}

/// Sync information
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SyncInfo {
    /// Latest block hash
    pub latest_block_hash: Hash,

    /// Latest app hash
    pub latest_app_hash: Hash,

    /// Latest block height
    pub latest_block_height: block::Height,

    /// Latest block time
    pub latest_block_time: Timestamp,

    /// Are we catching up?
    pub catching_up: bool,
}
