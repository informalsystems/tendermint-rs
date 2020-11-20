//! Error types

pub use eyre::{Report, Result};
use thiserror::Error;

/// Kinds of errors
#[derive(Copy, Clone, Debug, Error, Eq, PartialEq)]
pub enum Error {
    /// Cryptographic operation failed
    #[error("cryptographic error")]
    CryptoError,

    /// Malformatted or otherwise invalid cryptographic key
    #[error("invalid key")]
    InvalidKey,

    /// Network protocol-related errors
    #[error("protocol error")]
    ProtocolError,
}
