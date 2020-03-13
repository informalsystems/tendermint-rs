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
}

impl Kind {
    /// Add additional context.
    pub fn context(self, source: impl Into<BoxError>) -> Context<Kind> {
        Context::new(self, Some(source.into()))
    }
}
