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
            // if last_commit is Commit::Default, it is considered nil by Go.
            let last_commit = value
                .last_commit
                .map(TryInto::try_into)
                .transpose()?
                .filter(|c| c != &Commit::default());
            if last_commit.is_none() && header.height.value() != 1 {
                return Err(Error::invalid_block(
                    "last_commit is empty on non-first block".to_string(),
                ));
            }

            // Todo: Figure out requirements.
            // if last_commit.is_some() && header.height.value() == 1 {
            //    return Err(Kind::InvalidFirstBlock.context("last_commit is not null on first
            // height").into());
            //}

            Ok(Block {
                header,
                data: value.data.ok_or_else(Error::missing_data)?.txs,
                evidence: value.evidence.map(TryInto::try_into).transpose()?.unwrap_or_default(),
                last_commit,
            })
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
    /// constructor
    pub fn new(
        header: Header,
        data: Vec<Vec<u8>>,
        evidence: evidence::List,
        last_commit: Option<Commit>,
    ) -> Result<Self, Error> {
        if last_commit.is_none() && header.height.value() != 1 {
            return Err(Error::invalid_block(
                "last_commit is empty on non-first block".to_string(),
            ));
        }
        if last_commit.is_some() && header.height.value() == 1 {
            return Err(Error::invalid_block(
                "last_commit is filled on first block".to_string(),
            ));
        }
        Ok(Block {
            header,
            data,
            evidence,
            last_commit,
        })
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
