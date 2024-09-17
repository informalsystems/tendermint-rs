//! `/block_search` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};

use crate::dialect;
use crate::prelude::*;
use crate::request::RequestMessage;
use crate::serializers;
use crate::{Method, Order};

pub use super::{block, block_results};

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

impl RequestMessage for Request {
    fn method(&self) -> Method {
        Method::BlockSearch
    }
}

impl crate::Request<dialect::v0_34::Dialect> for Request {
    type Response = Response;
}

impl crate::SimpleRequest<dialect::v0_34::Dialect> for Request {
    type Output = Response;
}

impl crate::Request<dialect::v0_37::Dialect> for Request {
    type Response = Response;
}

impl crate::SimpleRequest<dialect::v0_37::Dialect> for Request {
    type Output = Response;
}

impl crate::Request<dialect::v0_38::Dialect> for Request {
    type Response = Response;
}

impl crate::SimpleRequest<dialect::v0_38::Dialect> for Request {
    type Output = Response;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    pub blocks: Vec<block::Response>,
    #[serde(with = "serializers::from_str")]
    pub total_count: u32,
}

impl crate::Response for Response {}

pub mod v0_38 {
    use super::*;

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct DialectResponse {
        pub blocks: Vec<block::v_038::DialectResponse>,

        #[serde(with = "serializers::from_str")]
        pub total_count: u32,
    }

    impl crate::Response for DialectResponse {}

    impl From<DialectResponse> for Response {
        fn from(response: DialectResponse) -> Self {
            Response {
                blocks: response.blocks.into_iter().map(Into::into).collect(),
                total_count: response.total_count,
            }
        }
    }
}
