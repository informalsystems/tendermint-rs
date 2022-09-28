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

use core::convert::TryFrom;

use tendermint_proto::{abci as pb, Protobuf};

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
