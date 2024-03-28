//! Blocks within the chains of a Tendermint network

mod block_id_flag;
mod commit;
pub mod commit_sig;
pub mod header;
mod height;
mod id;
mod meta;
pub mod parts;
mod round;
pub mod signed_header;
mod size;

use serde::{Deserialize, Serialize};
use tendermint_proto::v0_37::types::Block as RawBlock;

pub use self::{
    block_id_flag::BlockIdFlag,
    commit::*,
    commit_sig::*,
    header::Header,
    height::*,
    id::{Id, ParseId},
    meta::Meta,
    round::*,
    size::Size,
};
use crate::{error::Error, evidence, prelude::*};

/// Blocks consist of a header, transactions, votes (the commit), and a list of
/// evidence of malfeasance (i.e. signing conflicting votes).
///
/// <https://github.com/tendermint/spec/blob/d46cd7f573a2c6a2399fcab2cde981330aa63f37/spec/core/data_structures.md#block>
// Default serialization - all fields serialize; used by /block endpoint
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
#[serde(try_from = "RawBlock", into = "RawBlock")]
pub struct Block {
    /// Block header
    pub header: Header,

    /// Transaction data
    pub data: Vec<Vec<u8>>,

    /// Evidence of malfeasance
    pub evidence: evidence::List,

    /// Last commit
    pub last_commit: Option<Commit>,
}

tendermint_pb_modules! {
    use super::{Block, Header, Commit};
    use crate::{Error, prelude::*};
    use pb::types::Block as RawBlock;

    impl Protobuf<RawBlock> for Block {}

    impl TryFrom<RawBlock> for Block {
        type Error = Error;

        fn try_from(value: RawBlock) -> Result<Self, Self::Error> {
            let header: Header = value.header.ok_or_else(Error::missing_header)?.try_into()?;

            // If last_commit is the default Commit, it is considered nil by Go.
            let last_commit = value
                .last_commit
                .map(TryInto::try_into)
                .transpose()?
                .filter(|c| c != &Commit::default());

            Ok(Block::new_unchecked(
                header,
                value.data.ok_or_else(Error::missing_data)?.txs,
                value.evidence.map(TryInto::try_into).transpose()?.unwrap_or_default(),
                last_commit,
            ))
        }
    }

    impl From<Block> for RawBlock {
        fn from(value: Block) -> Self {
            use pb::types::Data as RawData;
            RawBlock {
                header: Some(value.header.into()),
                data: Some(RawData { txs: value.data }),
                evidence: Some(value.evidence.into()),
                last_commit: value.last_commit.map(Into::into),
            }
        }
    }
}

impl Block {
    /// Builds a new [`Block`], enforcing a couple invariants on the given [`Commit`]:
    /// - `last_commit` cannot be empty if the block is not the first one (ie. at height > 1)
    /// - `last_commit` must be empty if the block is the first one (ie. at height == 1)
    ///
    /// # Errors
    /// - If `last_commit` is empty on a non-first block
    /// - If `last_commit` is filled on the first block
    pub fn new(
        header: Header,
        data: Vec<Vec<u8>>,
        evidence: evidence::List,
        last_commit: Option<Commit>,
    ) -> Result<Self, Error> {
        let block = Self::new_unchecked(header, data, evidence, last_commit);
        block.validate()?;
        Ok(block)
    }

    /// Check that the following invariants hold for this block:
    ///
    /// - `last_commit` cannot be empty if the block is not the first one (ie. at height > 1)
    /// - `last_commit` must be empty if the block is the first one (ie. at height == 1)
    ///
    /// # Errors
    /// - If `last_commit` is empty on a non-first block
    /// - If `last_commit` is filled on the first block
    pub fn validate(&self) -> Result<(), Error> {
        if self.last_commit.is_none() && self.header.height.value() != 1 {
            return Err(Error::invalid_block(
                "last_commit is empty on non-first block".to_string(),
            ));
        }
        if self.last_commit.is_some() && self.header.height.value() == 1 {
            return Err(Error::invalid_block(
                "last_commit is filled on first block".to_string(),
            ));
        }

        Ok(())
    }

    /// Builds a new [`Block`], but does not enforce any invariants on the given [`Commit`].
    ///
    /// Use [`Block::new`] or [`Block::validate`] instead to enforce the following invariants, if necessary:
    /// - `last_commit` cannot be empty if the block is not the first one (ie. at height > 1)
    /// - `last_commit` must be empty if the block is the first one (ie. at height == 1)
    pub fn new_unchecked(
        header: Header,
        data: Vec<Vec<u8>>,
        evidence: evidence::List,
        last_commit: Option<Commit>,
    ) -> Self {
        Self {
            header,
            data,
            evidence,
            last_commit,
        }
    }

    /// Get header
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Get data
    pub fn data(&self) -> &Vec<Vec<u8>> {
        &self.data
    }

    /// Get evidence
    pub fn evidence(&self) -> &evidence::List {
        &self.evidence
    }

    /// Get last commit
    pub fn last_commit(&self) -> &Option<Commit> {
        &self.last_commit
    }
}
