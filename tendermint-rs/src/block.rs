//! Blocks within the chains of a Tendermint network

mod commit;
pub mod header;
mod height;
mod id;
mod meta;
pub mod parts;
mod size;

pub use self::{
    commit::LastCommit,
    header::Header,
    height::*,
    id::{Id, ParseId},
    meta::Meta,
    size::Size,
};
use crate::{abci::transaction, evidence};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Blocks consist of a header, transactions, votes (the commit), and a list of
/// evidence of malfeasance (i.e. signing conflicting votes).
///
/// <https://github.com/tendermint/tendermint/blob/master/docs/spec/blockchain/blockchain.md#block>
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Clone, Debug)]
pub struct Block {
    /// Block header
    pub header: Header,

    /// Transaction data
    pub data: transaction::Data,

    /// Evidence of malfeasance
    pub evidence: evidence::Data,

    /// Last commit
    pub last_commit: LastCommit,
}
