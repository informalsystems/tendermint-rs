//! `/block_search` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};

pub use super::{block, block_results};
use crate::{dialect::Dialect, prelude::*, serializers, Method, Order};

/// Request for searching for blocks by their BeginBlock and EndBlock events.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    pub query: String,
    #[serde(with = "serializers::from_str")]
    pub page: u32,
    #[serde(with = "serializers::from_str")]
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

impl<S: Dialect> crate::Request<S> for Request {
    type Response = Response;

    fn method(&self) -> Method {
        Method::BlockSearch
    }
}

impl<S: Dialect> crate::SimpleRequest<S> for Request {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    pub blocks: Vec<block::Response>,
    #[serde(with = "serializers::from_str")]
    pub total_count: u32,
}

impl crate::Response for Response {}
