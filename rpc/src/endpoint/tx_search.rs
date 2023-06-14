//! `/tx_search` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};

use crate::{
    dialect::{self, Dialect},
    prelude::*,
    request::RequestMessage,
    serializers, Method, Order,
};

pub use super::tx;

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

impl crate::Request<dialect::v0_34::Dialect> for Request {
    type Response = self::v0_34::DialectResponse;
}

impl crate::Request<dialect::v0_37::Dialect> for Request {
    type Response = Response;
}

impl<S: Dialect> crate::SimpleRequest<S> for Request
where
    Self: crate::Request<S>,
    Response: From<Self::Response>,
{
    type Output = Response;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Response {
    pub txs: Vec<tx::Response>,
    #[serde(with = "serializers::from_str")]
    pub total_count: u32,
}

impl crate::Response for Response {}

/// Serialization for /tx_search endpoint format in Tendermint 0.34
pub mod v0_34 {
    use super::{tx, Response};
    use crate::prelude::*;
    use crate::serializers;
    use serde::{Deserialize, Serialize};

    /// RPC dialect helper for serialization of the response.
    #[derive(Debug, Deserialize, Serialize)]
    pub struct DialectResponse {
        pub txs: Vec<tx::v0_34::DialectResponse>,
        #[serde(with = "serializers::from_str")]
        pub total_count: u32,
    }

    impl crate::Response for DialectResponse {}

    impl From<DialectResponse> for Response {
        fn from(msg: DialectResponse) -> Self {
            Self {
                txs: msg.txs.into_iter().map(Into::into).collect(),
                total_count: msg.total_count,
            }
        }
    }
}
