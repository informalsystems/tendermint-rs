//! Blocks within the chains of a Tendermint network

mod commit;
pub mod commit_sig;
pub mod header;
mod height;
mod id;
mod meta;
pub mod parts;
pub mod signed_header;
mod size;

pub use self::{
    commit::*,
    commit_sig::*,
    header::Header,
    height::*,
    id::{Id, ParseId},
    meta::Meta,
    size::Size,
};
use crate::{abci::transaction, evidence, serializers};
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
        pub height: Height,
        pub round: u64,
        #[serde(deserialize_with = "serializers::parse_non_empty_block_id")]
        pub block_id: Option<Id>,
        pub signatures: Option<CommitSigs>,
    }

    let commit = TmpCommit::deserialize(deserializer)?;
    if commit.block_id.is_none() || commit.signatures.is_none() {
        Ok(None)
    } else {
        Ok(Some(Commit {
            height: commit.height,
            round: commit.round,
            block_id: commit.block_id.unwrap(),
            signatures: commit.signatures.unwrap(),
        }))
    }
}
