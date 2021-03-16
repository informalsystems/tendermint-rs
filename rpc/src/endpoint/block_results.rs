//! `/block_results` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};

use tendermint::{abci, block, consensus, validator};

/// Get ABCI results at a given height.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    /// Height of the block to request.
    ///
    /// If no height is provided, it will fetch results for the latest block.
    pub height: Option<block::Height>,
}

impl Request {
    /// Create a new request for information about a particular block
    pub fn new(height: block::Height) -> Self {
        Self {
            height: Some(height),
        }
    }
}

impl crate::Request for Request {
    type Response = Response;

    fn method(&self) -> crate::Method {
        crate::Method::BlockResults
    }
}

impl crate::SimpleRequest for Request {}

/// ABCI result response.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Block height
    pub height: block::Height,

    /// Txs results (might be explicit null)
    pub txs_results: Option<Vec<abci::DeliverTx>>,

    /// Finalize block events (might be explicit null)
    pub finalize_block_events: Option<Vec<abci::Event>>,

    /// Validator updates (might be explicit null)
    #[serde(deserialize_with = "abci::responses::deserialize_validator_updates")]
    pub validator_updates: Vec<validator::Update>,

    /// New consensus params (might be explicit null)
    pub consensus_param_updates: Option<consensus::Params>,
}

impl crate::Response for Response {}
