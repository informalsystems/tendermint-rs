//! `/block_results` endpoint JSONRPC wrapper

use crate::{abci, block, rpc, validator, consensus};
use serde::{Deserialize, Serialize};

/// Get ABCI results at a given height.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    /// Height of the block to request.
    ///
    /// If no height is provided, it will fetch results for the latest block.
    height: Option<block::Height>,
}

impl Request {
    /// Create a new request for information about a particular block
    pub fn new(height: block::Height) -> Self {
        Self {
            height: Some(height),
        }
    }
}

impl rpc::Request for Request {
    type Response = Response;

    fn method(&self) -> rpc::Method {
        rpc::Method::BlockResults
    }
}

/// ABCI result response.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Block height
    pub height: block::Height,

    /// Txs results
    pub txs_results: abci::DeliverTx,

    /// Begin block events
    pub begin_block_events: abci::Event,

    /// End block events
    pub end_block_events: abci::Event,

    /// Validator updates
    #[serde(deserialize_with = "deserialize_validator_updates")]
    pub validator_updates: Vec<validator::Update>,

    /// New consensus params
    pub consensus_param_updates: Option<consensus::Params>,
}

impl rpc::Response for Response {}
