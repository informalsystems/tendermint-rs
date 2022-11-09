use crate::abci::Code;
use crate::prelude::*;

#[doc = include_str!("../doc/response-setoption.md")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SetOption {
    pub code: Code,
    pub log: String,
    pub info: String,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

use core::convert::TryFrom;

use tendermint_proto::{abci as pb, Protobuf};

impl From<SetOption> for pb::ResponseSetOption {
    fn from(message: SetOption) -> Self {
        Self {
            code: message.code.into(),
            log: message.log,
            info: message.info,
        }
    }
}

impl TryFrom<pb::ResponseSetOption> for SetOption {
    type Error = crate::Error;

    fn try_from(message: pb::ResponseSetOption) -> Result<Self, Self::Error> {
        Ok(Self {
            code: message.code.into(),
            log: message.log,
            info: message.info,
        })
    }
}

impl Protobuf<pb::ResponseSetOption> for SetOption {}
