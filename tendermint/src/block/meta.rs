//! Block metadata

use super::{Header, Id};
use crate::error::Error;
use crate::prelude::*;
use core::convert::{TryFrom, TryInto};
use serde::{Deserialize, Serialize, Serializer};
use tendermint_proto::types::BlockMeta as RawMeta;

/// Block metadata - Todo: implement constructor and getters
#[derive(Deserialize, Clone, Debug)]
#[serde(try_from = "RawMeta")]
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

impl TryFrom<Meta> for RawMeta {
    type Error = Error;

    fn try_from(value: Meta) -> Result<Self, Error> {
        Ok(RawMeta {
            block_id: Some(value.block_id.into()),
            block_size: value.block_size,
            header: Some(value.header.try_into()?),
            num_txs: value.num_txs,
        })
    }
}

impl Serialize for Meta {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let raw: RawMeta = self.clone().try_into().map_err(serde::ser::Error::custom)?;

        raw.serialize(serializer)
    }
}
