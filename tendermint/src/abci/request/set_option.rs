use crate::prelude::*;

#[doc = include_str!("../doc/request-setoption.md")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SetOption {
    pub key: String,
    pub value: String,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

// The SetOption request has been removed after 0.34.

use tendermint_proto::v0_34::abci as pb;
use tendermint_proto::Protobuf;

impl From<SetOption> for pb::RequestSetOption {
    fn from(message: SetOption) -> Self {
        Self {
            key: message.key,
            value: message.value,
        }
    }
}

impl TryFrom<pb::RequestSetOption> for SetOption {
    type Error = crate::Error;

    fn try_from(message: pb::RequestSetOption) -> Result<Self, Self::Error> {
        Ok(Self {
            key: message.key,
            value: message.value,
        })
    }
}

impl Protobuf<pb::RequestSetOption> for SetOption {}
