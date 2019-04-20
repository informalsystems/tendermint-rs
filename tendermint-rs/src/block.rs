//! Blocks within the chains of a Tendermint network

pub mod header;
mod height;
mod id;
mod meta;
pub mod parts;
mod size;

pub use self::{
    header::Header,
    height::*,
    id::{Id, ParseId},
    meta::Meta,
    size::Size,
};
use crate::{commit::LastCommit, evidence, transaction};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Block data
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Clone, Debug)]
pub struct Block {
    /// Block header
    pub header: Header,

    /// Data (i.e. transactions)
    pub data: transaction::Collection,

    /// Evidence of Byzantine behavior
    pub evidence: evidence::Collection,

    /// Last commit
    pub last_commit: LastCommit,
}
