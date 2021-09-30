//! `/abci_info` endpoint JSON-RPC wrapper

use serde::{Deserialize, Serialize};

use core::convert::{TryFrom, TryInto};
use tendermint::block;
use tendermint::Error;
use tendermint_proto::abci::ResponseInfo;

use crate::prelude::*;

/// Request ABCI information from a node
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Request;

impl crate::Request for Request {
    type Response = Response;

    fn method(&self) -> crate::Method {
        crate::Method::AbciInfo
    }
}

impl crate::SimpleRequest for Request {}

/// ABCI information response
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Response {
    /// ABCI info
    pub response: AbciInfo,
}

impl crate::Response for Response {}

/// ABCI information
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(default, try_from = "ResponseInfo", into = "ResponseInfo")]
pub struct AbciInfo {
    /// Name of the application
    pub data: String,

    /// Version
    pub version: String,

    /// App version
    pub app_version: u64,

    /// Last block height
    pub last_block_height: block::Height,

    /// Last app hash for the block
    pub last_block_app_hash: Vec<u8>,
}

impl TryFrom<ResponseInfo> for AbciInfo {
    type Error = Error;

    fn try_from(value: ResponseInfo) -> Result<Self, Self::Error> {
        Ok(AbciInfo {
            data: value.data,
            version: value.version,
            app_version: value.app_version,
            last_block_height: value.last_block_height.try_into()?,
            last_block_app_hash: value.last_block_app_hash,
        })
    }
}

impl From<AbciInfo> for ResponseInfo {
    fn from(value: AbciInfo) -> Self {
        ResponseInfo {
            data: value.data,
            version: value.version,
            app_version: value.app_version,
            last_block_height: value.last_block_height.into(),
            last_block_app_hash: value.last_block_app_hash,
        }
    }
}
