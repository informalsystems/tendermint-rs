use crate::prelude::*;

use bytes::Bytes;

use crate::block;

use super::super::types::{Evidence, LastCommitInfo};

// bring into scope for doc links
#[allow(unused)]
use super::DeliverTx;

#[doc = include_str!("../doc/request-beginblock.md")]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BeginBlock {
    /// The block's hash.
    ///
    /// This can be derived from the block header.
    pub hash: Bytes,
    /// The block header.
    pub header: block::Header,
    /// Information about the last commit.
    ///
    /// This includes the round, the list of validators, and which validators
    /// signed the last block.
    pub last_commit_info: LastCommitInfo,
    /// Evidence of validator misbehavior.
    pub byzantine_validators: Vec<Evidence>,
}

// =============================================================================
// Protobuf conversions
// =============================================================================

// XXX(hdevalence): these all use &'static str for now, this should be fixed
// to align with the crate's error-handling strategy.

use core::convert::{TryFrom, TryInto};
use tendermint_proto::abci as pb;
use tendermint_proto::Protobuf;

impl From<BeginBlock> for pb::RequestBeginBlock {
    fn from(begin_block: BeginBlock) -> Self {
        Self {
            hash: begin_block.hash,
            header: Some(begin_block.header.into()),
            last_commit_info: Some(begin_block.last_commit_info.into()),
            byzantine_validators: begin_block
                .byzantine_validators
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl TryFrom<pb::RequestBeginBlock> for BeginBlock {
    type Error = crate::Error;

    fn try_from(begin_block: pb::RequestBeginBlock) -> Result<Self, Self::Error> {
        Ok(Self {
            hash: begin_block.hash,
            header: begin_block.header.ok_or("missing header")?.try_into()?,
            last_commit_info: begin_block
                .last_commit_info
                .ok_or("missing last commit info")?
                .try_into()?,
            byzantine_validators: begin_block
                .byzantine_validators
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl Protobuf<pb::RequestBeginBlock> for BeginBlock {}
