//! Blocks within the chains of a Tendermint network

mod commit;
pub mod header;
mod height;
mod id;
mod meta;
pub mod parts;
mod size;

pub use self::{
    commit::*,
    header::{parse_non_empty_block_id, Header},
    height::*,
    id::{Id, ParseId},
    meta::Meta,
    size::Size,
};
use crate::{abci::transaction, evidence};
use serde::{Deserialize, Deserializer, Serialize};

/// Blocks consist of a header, transactions, votes (the commit), and a list of
/// evidence of malfeasance (i.e. signing conflicting votes).
///
/// <https://github.com/tendermint/tendermint/blob/master/docs/spec/blockchain/blockchain.md#block>
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Block {
    /// Block header
    pub header: Header,

    /// Transaction data
    pub data: transaction::Data,

    /// Evidence of malfeasance
    pub evidence: evidence::Data,

    /// Last commit
    #[serde(deserialize_with = "parse_non_empty_commit")]
    pub last_commit: Option<Commit>,
}

pub(crate) fn parse_non_empty_commit<'de, D>(deserializer: D) -> Result<Option<Commit>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct TmpCommit {
        #[serde(deserialize_with = "parse_non_empty_block_id")]
        block_id: Option<Id>,
        precommits: Option<Precommits>,
    }

    let commit = TmpCommit::deserialize(deserializer)?;
    if commit.block_id.is_none() || commit.precommits.is_none() {
        Ok(None)
    } else {
        Ok(Some(Commit {
            block_id: commit.block_id.unwrap(),
            precommits: commit.precommits.unwrap(),
        }))
    }
}
