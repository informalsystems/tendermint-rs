use crate::{block, prelude::*, AppHash};
use tendermint_proto::v0_37::abci as pb;

use serde::{Deserialize, Serialize};

#[doc = include_str!("../doc/response-info.md")]
#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(default, try_from = "pb::ResponseInfo", into = "pb::ResponseInfo")]
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
    pub last_block_app_hash: AppHash,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

tendermint_pb_modules! {
    use super::Info;

    impl From<Info> for pb::abci::ResponseInfo {
        fn from(info: Info) -> Self {
            Self {
                data: info.data,
                version: info.version,
                app_version: info.app_version,
                last_block_height: info.last_block_height.into(),
                last_block_app_hash: info.last_block_app_hash.into(),
            }
        }
    }

    impl TryFrom<pb::abci::ResponseInfo> for Info {
        type Error = crate::Error;

        fn try_from(info: pb::abci::ResponseInfo) -> Result<Self, Self::Error> {
            Ok(Self {
                data: info.data,
                version: info.version,
                app_version: info.app_version,
                last_block_height: info.last_block_height.try_into()?,
                last_block_app_hash: info.last_block_app_hash.try_into()?,
            })
        }
    }

    impl Protobuf<pb::abci::ResponseInfo> for Info {}
}
