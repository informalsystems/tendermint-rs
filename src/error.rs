//! Error types

use crate::{chain, prost};
use std::{
    any::Any,
    fmt::{self, Display},
    io,
};
use tendermint::amino_types::validate::ValidationError;

/// Error type
#[derive(Debug)]
pub struct Error(abscissa_core::Error<ErrorKind>);

impl Error {
    /// Create an error from a panic
    pub fn from_panic(msg: &dyn Any) -> Self {
        if let Some(e) = msg.downcast_ref::<String>() {
            err!(ErrorKind::PanicError, e)
        } else if let Some(e) = msg.downcast_ref::<&str>() {
            err!(ErrorKind::PanicError, e)
        } else {
            err!(ErrorKind::PanicError, "unknown cause")
        }
        .into()
    }
}

/// Kinds of errors
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
    /// Access denied
    #[fail(display = "access denied")]
    #[cfg(feature = "yubihsm")]
    AccessError,

    /// Error in configuration file
    #[fail(display = "config error")]
    ConfigError,

    /// KMS internal panic
    #[fail(display = "internal crash")]
    PanicError,

    /// Cryptographic operation failed
    #[fail(display = "cryptographic error")]
    CryptoError,

    /// Error running a subcommand to update chain state
    #[fail(display = "subcommand hook failed")]
    HookError,

    /// Malformatted or otherwise invalid cryptographic key
    #[fail(display = "invalid key")]
    InvalidKey,

    /// Validation of consensus message failed
    #[fail(display = "invalid consensus message")]
    InvalidMessageError,

    /// Input/output error
    #[fail(display = "I/O error")]
    IoError,

    /// Parse error
    #[fail(display = "parse error")]
    ParseError,

    /// Network protocol-related errors
    #[fail(display = "protocol error")]
    ProtocolError,

    /// Serialization error
    #[fail(display = "serialization error")]
    SerializationError,

    /// Signing operation failed
    #[fail(display = "signing operation failed")]
    SigningError,

    /// Verification operation failed
    #[fail(display = "verification failed")]
    VerificationError,

    /// Signature invalid
    #[fail(display = "attempted double sign")]
    DoubleSign,

    ///Request a Signature above max height
    #[fail(display = "requested signature above stop height")]
    ExceedMaxHeight,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error(abscissa_core::Error::new(kind, None))
    }
}

impl From<abscissa_core::Error<ErrorKind>> for Error {
    fn from(other: abscissa_core::Error<ErrorKind>) -> Self {
        Error(other)
    }
}

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Self {
        err!(ErrorKind::IoError, other).into()
    }
}

impl From<prost::DecodeError> for Error {
    fn from(other: prost::DecodeError) -> Self {
        err!(ErrorKind::ProtocolError, other).into()
    }
}

impl From<prost::EncodeError> for Error {
    fn from(other: prost::EncodeError) -> Self {
        err!(ErrorKind::ProtocolError, other).into()
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(other: serde_json::error::Error) -> Self {
        err!(ErrorKind::SerializationError, other).into()
    }
}

impl From<tendermint::Error> for Error {
    fn from(other: tendermint::error::Error) -> Self {
        let kind = match other.kind() {
            tendermint::ErrorKind::Crypto => ErrorKind::CryptoError,
            tendermint::ErrorKind::InvalidKey => ErrorKind::InvalidKey,
            tendermint::ErrorKind::Io => ErrorKind::IoError,
            tendermint::ErrorKind::Protocol => ErrorKind::ProtocolError,
            tendermint::ErrorKind::Length
            | tendermint::ErrorKind::Parse
            | tendermint::ErrorKind::OutOfRange => ErrorKind::ParseError,
            tendermint::ErrorKind::SignatureInvalid => ErrorKind::VerificationError,
        };

        abscissa_core::Error::new(kind, other.msg().map(|s| s.to_owned())).into()
    }
}

impl From<ValidationError> for Error {
    fn from(other: ValidationError) -> Self {
        err!(ErrorKind::InvalidMessageError, other).into()
    }
}

impl From<chain::state::StateError> for Error {
    fn from(other: chain::state::StateError) -> Self {
        err!(ErrorKind::DoubleSign, other).into()
    }
}
