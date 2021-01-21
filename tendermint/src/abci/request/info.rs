//! ABCI info request.

use crate::abci::request::{Request, RequestInner};
use crate::abci::response;
use crate::{Error, Kind};
use std::convert::TryFrom;
use tendermint_proto::abci::RequestInfo;
use tendermint_proto::Protobuf;

/// Allows a Tendermint node to provide information about itself to the ABCI
/// server, in exchange for information about the server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Info {
    /// Tendermint software semantic version.
    pub version: String,
    /// Tendermint block protocol version.
    pub block_version: u64,
    /// Tendermint P2P protocol version.
    pub p2p_version: u64,
    /// Tendermint ABCI version.
    pub abci_version: u64,
}

impl Default for Info {
    fn default() -> Self {
        Self {
            version: "".to_string(),
            block_version: 0,
            p2p_version: 0,
            abci_version: 0,
        }
    }
}

impl Protobuf<RequestInfo> for Info {}

impl TryFrom<RequestInfo> for Info {
    type Error = Error;

    fn try_from(value: RequestInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            version: value.version,
            block_version: value.block_version,
            p2p_version: value.p2p_version,
            abci_version: 0,
        })
    }
}

impl From<Info> for RequestInfo {
    fn from(value: Info) -> Self {
        Self {
            version: value.version,
            block_version: value.block_version,
            p2p_version: value.p2p_version,
        }
    }
}

impl RequestInner for Info {
    type Response = response::Info;
}

impl TryFrom<Request> for Info {
    type Error = Error;

    fn try_from(value: Request) -> Result<Self, Self::Error> {
        match value {
            Request::Info(r) => Ok(r),
            _ => Err(Kind::UnexpectedAbciRequestType("Info".to_owned(), value).into()),
        }
    }
}

impl From<Info> for Request {
    fn from(value: Info) -> Self {
        Self::Info(value)
    }
}
