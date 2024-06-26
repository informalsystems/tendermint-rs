//! `/genesis_chunked` endpoint JSON-RPC wrapper

use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};
use tendermint_proto::serializers;

use crate::{dialect::Dialect, request::RequestMessage};

/// Get the genesis state in multiple chunks for the current chain
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    pub chunk: Option<String>,
}

impl Request {
    pub fn new(chunk: Option<String>) -> Self {
        Self { chunk }
    }
}

impl RequestMessage for Request {
    fn method(&self) -> crate::Method {
        crate::Method::GenesisChunked
    }
}

impl<S> crate::Request<S> for Request
where
    S: Dialect,
{
    type Response = Response;
}

impl<S> crate::SimpleRequest<S> for Request
where
    S: Dialect,
{
    type Output = Response;
}

/// Block responses
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    #[serde(with = "serializers::from_str")]
    pub chunk: u64,
    #[serde(with = "serializers::from_str")]
    pub total: u64,
    #[serde(with = "serializers::bytes::base64string")]
    pub data: Vec<u8>,
}

impl crate::Response for Response {}
