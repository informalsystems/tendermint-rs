use prost;
use ring;
use signatory;
use std::io;

/// Kinds of errors
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum Error {
    /// Cryptographic operation failed
    #[fail(display = "cryptographic error")]
    CryptoError,

    /// Malformatted or otherwise invalid cryptographic key
    #[fail(display = "invalid key")]
    InvalidKey,

    /// Input/output error
    #[fail(display = "I/O error")]
    IoError,

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

impl From<io::Error> for Error {
    fn from(_other: io::Error) -> Self {
        Error::IoError
    }
}

impl From<prost::DecodeError> for Error {
    fn from(_other: prost::DecodeError) -> Self {
        Error::ProtocolError
    }
}

impl From<prost::EncodeError> for Error {
    fn from(_other: prost::EncodeError) -> Self {
        Error::ProtocolError
    }
}

impl From<ring::error::Unspecified> for Error {
    fn from(_other: ring::error::Unspecified) -> Self {
        Error::CryptoError
    }
}

impl From<signatory::Error> for Error {
    fn from(other: signatory::Error) -> Self {
        match other.kind() {
            signatory::ErrorKind::Io => Error::IoError,
            signatory::ErrorKind::KeyInvalid => Error::InvalidKey,
            signatory::ErrorKind::ParseError => Error::ProtocolError,
            signatory::ErrorKind::ProviderError => Error::SigningError,
            signatory::ErrorKind::SignatureInvalid => Error::VerificationError,
        }
    }
}
