//! Error types

use thiserror::Error;

/// Kinds of errors
#[derive(Copy, Clone, Debug, Error, Eq, PartialEq)]
pub enum Error {
    /// Cryptographic operation failed
    #[error("cryptographic error")]
    Crypto,

    /// Malformatted or otherwise invalid cryptographic key
    #[error("invalid key")]
    InvalidKey,

    /// Network protocol-related errors
    #[error("protocol error")]
    Protocol,
}
