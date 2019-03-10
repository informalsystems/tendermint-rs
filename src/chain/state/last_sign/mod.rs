//! State tracking for the last signature produced by the KMS

// TODO(tarcieri): consensus (e.g. Raft) around these values? State tracking service?
// See: https://github.com/tendermint/kms/issues/115

mod data;
mod error;

pub use self::{data::*, error::*};
