//! `/block_results` endpoint JSON-RPC wrapper

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tendermint::{abci, block, consensus, validator};

use crate::dialect::{self, Dialect};
use crate::prelude::*;
use crate::request::RequestMessage;
use crate::serializers;

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

impl RequestMessage for Request {
    fn method(&self) -> crate::Method {
        crate::Method::BlockResults
    }
}

impl<S: Dialect> crate::Request<S> for Request {
    type Response = DialectResponse<S::Event>;
}

impl<S: Dialect> crate::SimpleRequest<S> for Request {
    type Output = Response;
}

/// ABCI result response.
#[derive(Clone, Debug, Serialize)]
pub struct Response {
    /// Block height
    pub height: block::Height,

    /// Txs results (might be explicit null)
    pub txs_results: Option<Vec<abci::response::DeliverTx>>,

    /// Begin block events (might be explicit null)
    pub begin_block_events: Option<Vec<abci::Event>>,

    /// End block events (might be explicit null)
    pub end_block_events: Option<Vec<abci::Event>>,

    /// Validator updates (might be explicit null)
    pub validator_updates: Vec<validator::Update>,

    /// New consensus params (might be explicit null)
    pub consensus_param_updates: Option<consensus::Params>,
}

/// RPC dialect helper for serialization of the response.
#[derive(Debug, Serialize, Deserialize)]
pub struct DialectResponse<Ev> {
    /// Block height
    pub height: block::Height,

    /// Txs results (might be explicit null)
    pub txs_results: Option<Vec<dialect::DeliverTx<Ev>>>,

    /// Begin block events (might be explicit null)
    pub begin_block_events: Option<Vec<Ev>>,

    /// End block events (might be explicit null)
    pub end_block_events: Option<Vec<Ev>>,

    /// Validator updates (might be explicit null)
    #[serde(deserialize_with = "serializers::nullable::deserialize")]
    pub validator_updates: Vec<validator::Update>,

    /// New consensus params (might be explicit null)
    pub consensus_param_updates: Option<consensus::Params>,
}

impl<Ev> crate::Response for DialectResponse<Ev> where Ev: Serialize + DeserializeOwned {}

impl<Ev> From<DialectResponse<Ev>> for Response
where
    Ev: Into<abci::Event>,
{
    fn from(msg: DialectResponse<Ev>) -> Self {
        Response {
            height: msg.height,
            txs_results: msg
                .txs_results
                .map(|v| v.into_iter().map(Into::into).collect()),
            begin_block_events: msg
                .begin_block_events
                .map(|v| v.into_iter().map(Into::into).collect()),
            end_block_events: msg
                .end_block_events
                .map(|v| v.into_iter().map(Into::into).collect()),
            validator_updates: msg.validator_updates,
            consensus_param_updates: msg.consensus_param_updates,
        }
    }
}
