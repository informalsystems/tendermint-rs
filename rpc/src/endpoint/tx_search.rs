//! `/tx_search` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};

pub use super::tx;
use crate::{prelude::*, serializers, Method, Order};

/// Request for searching for transactions with their results.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request {
    pub query: String,
    pub prove: bool,
    #[serde(with = "serializers::from_str")]
    pub page: u32,
    #[serde(with = "serializers::from_str")]
    pub per_page: u8,
    pub order_by: Order,
}

impl Request {
    /// Constructor.
    pub fn new(
        query: impl ToString,
        prove: bool,
        page: u32,
        per_page: u8,
        order_by: Order,
    ) -> Self {
        Self {
            query: query.to_string(),
            prove,
            page,
            per_page,
            order_by,
        }
    }
}

impl crate::Request for Request {
    type Response = Response;

    fn method(&self) -> Method {
        Method::TxSearch
    }
}

impl crate::SimpleRequest for Request {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    pub txs: Vec<tx::Response>,
    #[serde(with = "serializers::from_str")]
    pub total_count: u32,
}

impl crate::Response for Response {}
