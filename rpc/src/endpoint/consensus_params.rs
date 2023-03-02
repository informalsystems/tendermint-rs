//! `/consensus_params` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};
use tendermint::block::Height;

use crate::{dialect::Dialect, request::RequestMessage};

/// Get the consensus parameters.
///
/// If no height is supplied, the latest consensus parameters will be returned.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    pub height: Option<Height>,
}

impl Request {
    /// Constructor with optional height.
    pub fn new(maybe_height: Option<Height>) -> Self {
        Self {
            height: maybe_height,
        }
    }
}

impl RequestMessage for Request {
    fn method(&self) -> crate::Method {
        crate::Method::ConsensusParams
    }
}

impl<S: Dialect> crate::Request<S> for Request {
    type Response = Response;
}

impl<S: Dialect> crate::SimpleRequest<S> for Request {
    type Output = Response;
}

/// Consensus parameters response.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Response {
    pub block_height: Height,
    pub consensus_params: tendermint::consensus::Params,
}

impl crate::Response for Response {}
