// Error types

use failure::{Backtrace, Context, Fail};
use prost;
use ring;
use signatory;
use std::{
    any::Any,
    error::Error as StdError,
    fmt::{self, Display},
    io,
};

/// Create a new error (of a given enum variant) with a formatted message
macro_rules! err {
    ($variant:ident, $msg:expr) => {
        ::error::Error::with_description(
            ::error::ErrorKind::$variant,
            $msg.to_string()
        )
    };
    ($variant:ident, $fmt:expr, $($arg:tt)+) => {
        ::error::Error::with_description(
            ::error::ErrorKind::$variant,
            format!($fmt, $($arg)+)
        )
    };
}

/// Error type
#[derive(Debug)]
pub struct Error {
    /// Contextual information about the error
    inner: Context<ErrorKind>,

    /// Optional description message
    description: Option<String>,
}

impl Error {
    /// Create a new error
    pub fn new(kind: ErrorKind) -> Self {
        Self {
            inner: Context::new(kind),
            description: None,
        }
    }

    /// Create a new error with the given description
    pub fn with_description(kind: ErrorKind, description: String) -> Self {
        Self {
            inner: Context::new(kind),
            description: Some(description),
        }
    }

    /// Obtain the inner `ErrorKind` for this error
    #[allow(dead_code)]
    pub fn kind(&self) -> ErrorKind {
        *self.inner.get_context()
    }

    /// Create an error from a panic
    pub fn from_panic(msg: &Any) -> Self {
        if let Some(e) = msg.downcast_ref::<String>() {
            err!(PanicError, e)
        } else if let Some(e) = msg.downcast_ref::<&str>() {
            err!(PanicError, e)
        } else {
            err!(PanicError, "unknown cause")
        }
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self::new(kind)
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Self {
        Self {
            inner,
            description: None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.description {
            Some(ref desc) => write!(f, "{}: {}", &self.inner, desc),
            None => Display::fmt(&self.inner, f),
        }
    }
}

/// Kinds of errors
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
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

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Self {
        err!(IoError, other)
    }
}

impl From<prost::DecodeError> for Error {
    fn from(other: prost::DecodeError) -> Self {
        err!(ProtocolError, other)
    }
}

impl From<prost::EncodeError> for Error {
    fn from(other: prost::EncodeError) -> Self {
        err!(ProtocolError, other)
    }
}

impl From<ring::error::Unspecified> for Error {
    fn from(other: ring::error::Unspecified) -> Self {
        err!(CryptoError, other)
    }
}

impl From<signatory::Error> for Error {
    fn from(other: signatory::Error) -> Self {
        let kind = match other.kind() {
            signatory::ErrorKind::Io => ErrorKind::IoError,
            signatory::ErrorKind::KeyInvalid => ErrorKind::InvalidKey,
            signatory::ErrorKind::ParseError => ErrorKind::ParseError,
            signatory::ErrorKind::ProviderError => ErrorKind::SigningError,
            signatory::ErrorKind::SignatureInvalid => ErrorKind::VerificationError,
        };

        Error::with_description(kind, other.description().to_owned())
    }
}
