//! `/broadcast_evidence`: broadcast an evidence.

use serde::{Deserialize, Serialize};
use tendermint::{evidence, Hash};

use crate::{dialect::Dialect, request::RequestMessage, Method};

/// `/broadcast_evidence`: broadcast an evidence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request<S: Dialect> {
    /// Evidence to broadcast
    pub evidence: S::Evidence,
}

impl<S: Dialect> Request<S> {
    /// Create a new evidence broadcast RPC request
    pub fn new(evidence: evidence::Evidence) -> Self {
        Request {
            evidence: evidence.into(),
        }
    }
}

impl<S: Dialect> RequestMessage for Request<S> {
    fn method(&self) -> Method {
        Method::BroadcastEvidence
    }
}

impl<S: Dialect> crate::Request<S> for Request<S> {
    type Response = Response;
}

impl<S: Dialect> crate::SimpleRequest<S> for Request<S> {
    type Output = Response;
}

/// Response from either an evidence broadcast request.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// Evidence hash
    #[serde(with = "crate::serializers::tm_hash_base64")]
    pub hash: Hash,
}

impl crate::Response for Response {}
