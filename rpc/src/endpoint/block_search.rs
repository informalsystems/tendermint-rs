//! `/block_search` endpoint JSON-RPC wrapper

pub use super::{block, block_results};

use crate::prelude::*;
use crate::{Method, Order};
use serde::{Deserialize, Serialize};

/// Request for searching for blocks by their BeginBlock and EndBlock events.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    pub query: String,
    #[serde(with = "tendermint_proto::serializers::from_str")]
    pub page: u32,
    #[serde(with = "tendermint_proto::serializers::from_str")]
    pub per_page: u8,
    pub order_by: Order,
}

impl Request {
    /// Constructor.
    pub fn new(query: impl ToString, page: u32, per_page: u8, order_by: Order) -> Self {
        Self {
            query: query.to_string(),
            page,
            per_page,
            order_by,
        }
    }
}

impl crate::Request for Request {
    type Response = Response;

    fn method(&self) -> Method {
        Method::BlockSearch
    }
}

impl crate::SimpleRequest for Request {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    pub blocks: Vec<block::Response>,
    #[serde(with = "tendermint_proto::serializers::from_str")]
    pub total_count: u32,
}

impl crate::Response for Response {}
