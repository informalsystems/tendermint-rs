use crate::{block, prelude::*, Error};

use bytes::Bytes;

#[doc = include_str!("../doc/response-info.md")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Info {
    /// Some arbitrary information.
    pub data: String,
    /// The application software semantic version.
    pub version: String,
    /// The application protocol version.
    pub app_version: u64,
    /// The latest block for which the app has called [`Commit`](super::super::Request::Commit).
    pub last_block_height: block::Height,
    /// The latest result of [`Commit`](super::super::Request::Commit).
    // XXX(hdevalence): fix this, should be apphash?
    pub last_block_app_hash: Bytes,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

use core::convert::TryFrom;
use tendermint_proto::abci as pb;
use tendermint_proto::Protobuf;

impl From<Info> for pb::ResponseInfo {
    fn from(info: Info) -> Self {
        Self {
            data: info.data,
            version: info.version,
            app_version: info.app_version,
            last_block_height: info.last_block_height.into(),
            last_block_app_hash: info.last_block_app_hash,
        }
    }
}

impl TryFrom<pb::ResponseInfo> for Info {
    type Error = Error;

    fn try_from(info: pb::ResponseInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            data: info.data,
            version: info.version,
            app_version: info.app_version,
            last_block_height: info.last_block_height.try_into()?,
            last_block_app_hash: info.last_block_app_hash,
        })
    }
}

impl Protobuf<pb::ResponseInfo> for Info {}
