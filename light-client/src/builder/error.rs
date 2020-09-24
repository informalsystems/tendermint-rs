//! Errors raised by the builder DSL

use tendermint::block::Height;
use tendermint::Hash;
use thiserror::Error;

use crate::components::io::IoError;

/// An error raised by the builder
pub type Error = anomaly::Error<Kind>;

/// The various error kinds raised by the builder
#[derive(Debug, Clone, Error, PartialEq)]
pub enum Kind {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] IoError),

    /// Height mismatch
    #[error("height mismatch: given = {given}, found = {found}")]
    HeightMismatch {
        /// Height of trusted header
        given: Height,
        /// Height of fetched header
        found: Height,
    },

    /// Hash mismatch
    #[error("hash mismatch: given = {given}, found = {found}")]
    HashMismatch {
        /// Hash of trusted header
        given: Hash,
        /// hash of fetched header
        found: Hash,
    },
}
