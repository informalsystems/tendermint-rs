use crate::prelude::*;

use bytes::Bytes;

#[doc = include_str!("../doc/response-info.md")]
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Info {
    /// Some arbitrary information.
    pub data: String,
    /// The application software semantic version.
    pub version: String,
    /// The application protocol version.
    pub app_version: u64,
    /// The latest block for which the app has called [`Commit`](super::super::Request::Commit).
    pub last_block_height: i64,
    /// The latest result of [`Commit`](super::super::Request::Commit).
    // XXX(hdevalence): fix this, should be apphash?
    pub last_block_app_hash: Bytes,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

// XXX(hdevalence): these all use &'static str for now, this should be fixed
// to align with the crate's error-handling strategy.

use core::convert::TryFrom;
use tendermint_proto::abci as pb;
use tendermint_proto::Protobuf;

impl From<Info> for pb::ResponseInfo {
    fn from(info: Info) -> Self {
        Self {
            data: info.data,
            version: info.version,
            app_version: info.app_version,
            last_block_height: info.last_block_height,
            last_block_app_hash: info.last_block_app_hash,
        }
    }
}

impl TryFrom<pb::ResponseInfo> for Info {
    type Error = &'static str;

    fn try_from(info: pb::ResponseInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            data: info.data,
            version: info.version,
            app_version: info.app_version,
            last_block_height: info.last_block_height,
            last_block_app_hash: info.last_block_app_hash,
        })
    }
}

impl Protobuf<pb::ResponseInfo> for Info {}
