//! Error types

use chrono;
#[cfg(feature = "secret-connection")]
use prost;
#[cfg(feature = "secret-connection")]
use signatory;
use std::{self, io};
use subtle_encoding;

/// Kinds of errors
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum Error {
    /// Cryptographic operation failed
    #[fail(display = "cryptographic error")]
    Crypto,

    /// Malformatted or otherwise invalid cryptographic key
    #[fail(display = "invalid key")]
    InvalidKey,

    /// Input/output error
    #[fail(display = "I/O error")]
    Io,

    /// Length incorrect or too long
    #[fail(display = "length error")]
    Length,

    /// Parse error
    #[fail(display = "parse error")]
    Parse,

    /// Network protocol-related errors
    #[fail(display = "protocol error")]
    Protocol,

    /// Value out-of-range
    #[fail(display = "value out of range")]
    OutOfRange,

    /// Signature invalid
    #[fail(display = "bad signature")]
    SignatureInvalid,
}

/// Result type with our error type already defined
pub type Result<T> = std::result::Result<T, Error>;

impl From<chrono::ParseError> for Error {
    fn from(_: chrono::ParseError) -> Error {
        Error::Parse
    }
}

impl From<io::Error> for Error {
    fn from(_other: io::Error) -> Self {
        Error::Io
    }
}

#[cfg(feature = "secret-connection")]
impl From<prost::DecodeError> for Error {
    fn from(_other: prost::DecodeError) -> Self {
        Error::Protocol
    }
}

#[cfg(feature = "secret-connection")]
impl From<prost::EncodeError> for Error {
    fn from(_other: prost::EncodeError) -> Self {
        Error::Protocol
    }
}

#[cfg(feature = "secret-connection")]
impl From<subtle_encoding::Error> for Error {
    fn from(_: subtle_encoding::Error) -> Error {
        Error::Parse
    }
}

#[cfg(feature = "secret-connection")]
impl From<signatory::Error> for Error {
    fn from(other: signatory::Error) -> Self {
        match other.kind() {
            signatory::ErrorKind::Io => Error::Io,
            signatory::ErrorKind::KeyInvalid => Error::InvalidKey,
            signatory::ErrorKind::ParseError => Error::Parse,
            signatory::ErrorKind::ProviderError => Error::Crypto,
            signatory::ErrorKind::SignatureInvalid => Error::SignatureInvalid,
        }
    }
}
