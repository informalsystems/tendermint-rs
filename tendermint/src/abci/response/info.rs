//! ABCI info response.

use crate::abci::response::{Response, ResponseInner};
use crate::{Error, Kind};
use std::convert::TryFrom;
use tendermint_proto::abci::ResponseInfo;
use tendermint_proto::Protobuf;

/// Allows the ABCI app to provide information about itself back to the
/// Tendermint node.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Info {
    /// Arbitrary (application-specific) information.
    pub data: String,
    /// Application software semantic version.
    pub version: String,
    /// Application protocol version.
    pub app_version: u64,
    /// Latest block for which the application has called Commit.
    pub last_block_height: i64,
    /// Latest result of Commit.
    pub last_block_app_hash: Vec<u8>,
}

impl Info {
    /// Constructor.
    pub fn new<S, V>(
        data: S,
        version: S,
        app_version: u64,
        last_block_height: i64,
        last_block_app_hash: V,
    ) -> Self
    where
        S: AsRef<str>,
        V: AsRef<[u8]>,
    {
        Self {
            data: data.as_ref().to_owned(),
            version: version.as_ref().to_owned(),
            app_version,
            last_block_height,
            last_block_app_hash: last_block_app_hash.as_ref().to_vec(),
        }
    }
}

impl Default for Info {
    fn default() -> Self {
        Self::new("", "", 0, 0, [])
    }
}

impl Protobuf<ResponseInfo> for Info {}

impl TryFrom<ResponseInfo> for Info {
    type Error = Error;

    fn try_from(value: ResponseInfo) -> Result<Self, Self::Error> {
        Ok(Self::new(
            value.data,
            value.version,
            value.app_version,
            value.last_block_height,
            value.last_block_app_hash,
        ))
    }
}

impl From<Info> for ResponseInfo {
    fn from(value: Info) -> Self {
        Self {
            data: value.data,
            version: value.version,
            app_version: value.app_version,
            last_block_height: value.last_block_height,
            last_block_app_hash: value.last_block_app_hash,
        }
    }
}

impl ResponseInner for Info {}

impl TryFrom<Response> for Info {
    type Error = Error;

    fn try_from(value: Response) -> Result<Self, Self::Error> {
        match value {
            Response::Info(res) => Ok(res),
            _ => Err(Kind::UnexpectedAbciResponseType("Info".to_owned(), value).into()),
        }
    }
}

impl From<Info> for Response {
    fn from(value: Info) -> Self {
        Self::Info(value)
    }
}
