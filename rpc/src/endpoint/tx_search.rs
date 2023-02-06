//! `/tx_search` endpoint JSON-RPC wrapper

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use tendermint::abci;

pub use super::tx;
use crate::{dialect::Dialect, prelude::*, request::RequestMessage, serializers, Method, Order};

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

impl RequestMessage for Request {
    fn method(&self) -> Method {
        Method::TxSearch
    }
}

impl<S: Dialect> crate::Request<S> for Request {
    type Response = DialectResponse<S::Event>;
}

impl<S: Dialect> crate::SimpleRequest<S> for Request {
    type Output = Response;
}

#[derive(Clone, Debug, Serialize)]
pub struct Response {
    pub txs: Vec<tx::Response>,
    pub total_count: u32,
}

/// RPC dialect helper for serialization of the response.
#[derive(Debug, Deserialize, Serialize)]
pub struct DialectResponse<Ev> {
    pub txs: Vec<tx::DialectResponse<Ev>>,
    #[serde(with = "serializers::from_str")]
    pub total_count: u32,
}

impl<Ev> crate::Response for DialectResponse<Ev> where Ev: Serialize + DeserializeOwned {}

impl<Ev> From<DialectResponse<Ev>> for Response
where
    Ev: Into<abci::Event>,
{
    fn from(msg: DialectResponse<Ev>) -> Self {
        Self {
            txs: msg.txs.into_iter().map(Into::into).collect(),
            total_count: msg.total_count,
        }
    }
}
