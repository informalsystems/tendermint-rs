//! `/block_results` endpoint JSON-RPC wrapper

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tendermint::{block, consensus, validator};

use crate::dialect::{DeliverTx, Dialect};
use crate::prelude::*;
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

impl<S: Dialect> crate::Request<S> for Request {
    type Response = Response<S::Event>;

    fn method(&self) -> crate::Method {
        crate::Method::BlockResults
    }
}

impl<S: Dialect> crate::SimpleRequest<S> for Request {}

/// ABCI result response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Response<Ev> {
    /// Block height
    pub height: block::Height,

    /// Txs results (might be explicit null)
    pub txs_results: Option<Vec<DeliverTx<Ev>>>,

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

impl<Ev> crate::Response for Response<Ev> where Ev: Serialize + DeserializeOwned {}
