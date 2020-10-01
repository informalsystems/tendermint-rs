//! Error types

use anomaly::{BoxError, Context};
use thiserror::Error;

/// Error type
pub type Error = BoxError;

/// Kinds of errors
#[derive(Clone, Eq, PartialEq, Debug, Error)]
pub enum Kind {
    /// Cryptographic operation failed
    #[error("cryptographic error")]
    Crypto,

    /// Malformatted or otherwise invalid cryptographic key
    #[error("invalid key")]
    InvalidKey,

    /// Input/output error
    #[error("I/O error")]
    Io,

    /// Length incorrect or too long
    #[error("length error")]
    Length,

    /// Parse error
    #[error("parse error")]
    Parse,

    /// Network protocol-related errors
    #[error("protocol error")]
    Protocol,

    /// Value out-of-range
    #[error("value out of range")]
    OutOfRange,

    /// Signature invalid
    #[error("bad signature")]
    SignatureInvalid,

    /// invalid Vote type
    #[error("invalid Type")]
    InvalidMessageType,

    /// Negative block height
    #[error("negative height")]
    NegativeHeight,

    /// Negative voting round
    #[error("negative round")]
    NegativeRound,

    /// Negative POL round
    #[error("negative pol round")]
    NegativePOLRound,

    /// Negative validator index in vote
    #[error("negative ValidatorIndex")]
    NegativeValidatorIndex,

    /// Wrong hash size in part_set_header
    #[error("Wrong hash: expected Hash size to be 32 bytes")]
    InvalidHashSize,

    /// No timestamp in vote
    #[error("no timestamp")]
    NoTimestamp,

    /// Invalid account ID length
    #[error("invalid account ID length")]
    InvalidAccountIDLength,

    /// Invalid signature ID length
    #[error("invalid signature ID length")]
    InvalidSignatureIDLength,

    /// Overflow during conversion
    #[error("Integer overflow")]
    IntegerOverflow,
}

impl Kind {
    /// Add additional context.
    pub fn context(self, source: impl Into<BoxError>) -> Context<Kind> {
        Context::new(self, Some(source.into()))
    }
}
