//! Block metadata

use super::{Header, Id};
use crate::error::Error;
use crate::prelude::*;
use core::convert::{TryFrom, TryInto};
use serde::{Deserialize, Serialize};
use tendermint_proto::types::BlockMeta as RawMeta;

/// Block metadata - Todo: implement constructor and getters
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(try_from = "RawMeta", into = "RawMeta")]
pub struct Meta {
    /// ID of the block
    pub block_id: Id,

    /// block size - Todo: make this robust (u63)
    pub block_size: i64,

    /// Header of the block
    pub header: Header,

    /// Number of transactions - Todo: make this robust (u63)
    pub num_txs: i64,
}

impl TryFrom<RawMeta> for Meta {
    type Error = Error;

    fn try_from(value: RawMeta) -> Result<Self, Self::Error> {
        Ok(Meta {
            block_id: value
                .block_id
                .ok_or_else(|| Error::invalid_block("no block_id".to_string()))?
                .try_into()?,
            block_size: value.block_size,
            header: value
                .header
                .ok_or_else(|| Error::invalid_block("no header".to_string()))?
                .try_into()?,
            num_txs: value.num_txs,
        })
    }
}

impl From<Meta> for RawMeta {
    fn from(value: Meta) -> Self {
        RawMeta {
            block_id: Some(value.block_id.into()),
            block_size: value.block_size,
            header: Some(value.header.into()),
            num_txs: value.num_txs,
        }
    }
}
