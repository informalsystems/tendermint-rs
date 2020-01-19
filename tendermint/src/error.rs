//! Error types

use failure::*;
use std::{
    fmt::{self, Display},
    io,
};
use {chrono, prost_amino, subtle_encoding};

/// Create a new error (of a given kind) with a formatted message
#[allow(unused_macros)]
macro_rules! err {
    ($kind:path, $msg:expr) => {
        $crate::error::Error::new(failure::Context::new($kind), Some($msg.to_string()))
    };
    ($kind:path, $fmt:expr, $($arg:tt)+) => {
        err!($kind, &format!($fmt, $($arg)+))
    };
}

/// Error type
#[derive(Debug)]
pub struct Error {
    /// Contextual information about the error
    inner: Context<ErrorKind>,

    /// Optional message to associate with the error
    msg: Option<String>,
}

impl Error {
    /// Create a new error from the given context and optional message
    pub fn new<C>(context: C, msg: Option<String>) -> Self
    where
        C: Into<Context<ErrorKind>>,
    {
        Self {
            inner: context.into(),
            msg,
        }
    }

    /// Obtain the error's `ErrorKind`
    pub fn kind(&self) -> &ErrorKind {
        self.inner.get_context()
    }

    /// Get the message associated with this error (if available)
    pub fn msg(&self) -> Option<&str> {
        self.msg.as_ref().map(AsRef::as_ref)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(msg) = &self.msg {
            write!(f, "{}: {}", self.kind(), msg)
        } else {
            write!(f, "{}", self.kind())
        }
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error::new(kind, None)
    }
}

impl From<chrono::ParseError> for Error {
    fn from(err: chrono::ParseError) -> Error {
        err!(ErrorKind::Parse, err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        err!(ErrorKind::Io, err)
    }
}

impl From<prost_amino::DecodeError> for Error {
    fn from(err: prost_amino::DecodeError) -> Self {
        err!(ErrorKind::Parse, err)
    }
}

impl From<prost_amino::EncodeError> for Error {
    fn from(err: prost_amino::EncodeError) -> Self {
        err!(ErrorKind::Parse, err)
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(err: serde_json::error::Error) -> Self {
        err!(ErrorKind::Parse, err)
    }
}

impl From<signatory::signature::Error> for Error {
    fn from(err: signatory::signature::Error) -> Self {
        use std::error::Error as _;

        if let Some(source) = err.source() {
            err!(ErrorKind::Crypto, "signature error: {}", source)
        } else {
            err!(ErrorKind::Crypto, "signature error")
        }
    }
}

impl From<subtle_encoding::Error> for Error {
    fn from(err: subtle_encoding::Error) -> Error {
        err!(ErrorKind::Parse, err)
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        err!(ErrorKind::Parse, err)
    }
}

/// Kinds of errors
#[derive(Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
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
