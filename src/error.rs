// Error types

use abscissa::Error;
use prost;
use signatory;
use std::{
    any::Any,
    error::Error as StdError,
    fmt::{self, Display},
    io,
};
use tm_secret_connection;
#[cfg(feature = "yubihsm")]
use yubihsm;

/// Error type
#[derive(Debug)]
pub struct KmsError(Error<KmsErrorKind>);

impl KmsError {
    /// Create an error from a panic
    pub fn from_panic(msg: &Any) -> Self {
        if let Some(e) = msg.downcast_ref::<String>() {
            err!(KmsErrorKind::PanicError, e)
        } else if let Some(e) = msg.downcast_ref::<&str>() {
            err!(KmsErrorKind::PanicError, e)
        } else {
            err!(KmsErrorKind::PanicError, "unknown cause")
        }.into()
    }
}

/// Kinds of errors
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum KmsErrorKind {
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

    /// Malformatted or otherwise invalid cryptographic key
    #[fail(display = "invalid key")]
    InvalidKey,

    /// Input/output error
    #[fail(display = "I/O error")]
    IoError,

    /// Parse error
    #[fail(display = "parse error")]
    ParseError,

    /// Network protocol-related errors
    #[fail(display = "protocol error")]
    ProtocolError,

    /// Signing operation failed
    #[fail(display = "signing operation failed")]
    SigningError,

    /// Verification operation failed
    #[fail(display = "verification failed")]
    VerificationError,
}

impl Display for KmsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<Error<KmsErrorKind>> for KmsError {
    fn from(other: Error<KmsErrorKind>) -> Self {
        KmsError(other)
    }
}

impl From<io::Error> for KmsError {
    fn from(other: io::Error) -> Self {
        err!(KmsErrorKind::IoError, other).into()
    }
}

impl From<prost::DecodeError> for KmsError {
    fn from(other: prost::DecodeError) -> Self {
        err!(KmsErrorKind::ProtocolError, other).into()
    }
}

impl From<prost::EncodeError> for KmsError {
    fn from(other: prost::EncodeError) -> Self {
        err!(KmsErrorKind::ProtocolError, other).into()
    }
}

impl From<signatory::Error> for KmsError {
    fn from(other: signatory::Error) -> Self {
        let kind = match other.kind() {
            signatory::ErrorKind::Io => KmsErrorKind::IoError,
            signatory::ErrorKind::KeyInvalid => KmsErrorKind::InvalidKey,
            signatory::ErrorKind::ParseError => KmsErrorKind::ParseError,
            signatory::ErrorKind::ProviderError => KmsErrorKind::SigningError,
            signatory::ErrorKind::SignatureInvalid => KmsErrorKind::VerificationError,
        };

        Error::new(kind, Some(other.description().to_owned())).into()
    }
}

impl From<tm_secret_connection::Error> for KmsError {
    fn from(other: tm_secret_connection::Error) -> Self {
        let kind = match other {
            tm_secret_connection::Error::CryptoError => KmsErrorKind::CryptoError,
            tm_secret_connection::Error::InvalidKey => KmsErrorKind::InvalidKey,
            tm_secret_connection::Error::IoError => KmsErrorKind::IoError,
            tm_secret_connection::Error::ProtocolError => KmsErrorKind::ProtocolError,
            tm_secret_connection::Error::SigningError => KmsErrorKind::SigningError,
            tm_secret_connection::Error::VerificationError => KmsErrorKind::VerificationError,
        };

        Error::new(kind, None).into()
    }
}

#[cfg(feature = "yubihsm")]
impl From<yubihsm::connector::ConnectionError> for KmsError {
    fn from(other: yubihsm::connector::ConnectionError) -> Self {
        use yubihsm::connector::ConnectionErrorKind;

        let kind = match other.kind() {
            ConnectionErrorKind::AddrInvalid => KmsErrorKind::ConfigError,
            ConnectionErrorKind::AccessDenied => KmsErrorKind::AccessError,
            ConnectionErrorKind::IoError
            | ConnectionErrorKind::ConnectionFailed
            | ConnectionErrorKind::DeviceBusyError
            | ConnectionErrorKind::RequestError
            | ConnectionErrorKind::ResponseError
            | ConnectionErrorKind::UsbError => KmsErrorKind::IoError,
        };

        Error::new(kind, Some(other.description().to_owned())).into()
    }
}
