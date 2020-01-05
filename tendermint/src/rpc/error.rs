//! JSONRPC error types

use failure::Fail;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{self, Display};

/// Tendermint RPC errors
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Error {
    /// Error code
    code: Code,

    /// Error message
    message: String,

    /// Additional data about the error
    data: Option<String>,
}

impl Error {
    /// Create a new RPC error
    pub fn new(code: Code, data: Option<String>) -> Self {
        let message = code.to_string();

        Self {
            code,
            message,
            data,
        }
    }

    /// Create a low-level HTTP error
    pub fn http_error(message: impl Into<String>) -> Self {
        Self {
            code: Code::HttpError,
            message: message.into(),
            data: None,
        }
    }

    /// Create a new invalid parameter error
    pub fn invalid_params(data: &str) -> Self {
        Self::new(Code::InvalidParams, Some(data.to_string()))
    }

    /// Create a new method-not-found error
    pub fn method_not_found(name: &str) -> Self {
        Self::new(Code::MethodNotFound, Some(name.to_string()))
    }

    /// Create a new parse error
    pub fn parse_error<E>(error: E) -> Self
    where
        E: Display,
    {
        Self::new(Code::ParseError, Some(error.to_string()))
    }

    /// Create a new server error
    pub fn server_error<D>(data: D) -> Self
    where
        D: Display,
    {
        Self::new(Code::ServerError, Some(data.to_string()))
    }

    /// Obtain the `rpc::error::Code` for this error
    pub const fn code(&self) -> Code {
        self.code
    }

    /// Borrow the error message (if available)
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Optional additional error message (if available)
    pub fn data(&self) -> Option<&str> {
        self.data.as_ref().map(AsRef::as_ref)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.data {
            Some(data) => write!(
                f,
                "{}: {} (code: {})",
                self.message,
                data,
                self.code.value()
            ),
            None => write!(f, "{} (code: {})", self.message, self.code.value()),
        }
    }
}

impl Fail for Error {
    fn name(&self) -> Option<&str> {
        self.code.name()
    }
}

impl From<http::Error> for Error {
    fn from(http_error: http::Error) -> Self {
        Self::http_error(http_error.to_string())
    }
}

impl From<hyper::Error> for Error {
    fn from(hyper_error: hyper::Error) -> Self {
        Self::http_error(hyper_error.to_string())
    }
}

/// Tendermint RPC error codes.
///
/// See `func RPC*Error()` definitions in:
/// <https://github.com/tendermint/tendermint/blob/master/rpc/lib/types/types.go>
#[derive(Copy, Clone, Debug, Eq, Fail, Hash, PartialEq, PartialOrd, Ord)]
pub enum Code {
    /// Low-level HTTP error
    #[fail(display = "HTTP error")]
    HttpError,

    /// Parse error i.e. invalid JSON (-32700)
    #[fail(display = "Parse error. Invalid JSON")]
    ParseError,

    /// Invalid request (-32600)
    #[fail(display = "Invalid Request")]
    InvalidRequest,

    /// Method not found error (-32601)
    #[fail(display = "Method not found")]
    MethodNotFound,

    /// Invalid parameters (-32602)
    #[fail(display = "Invalid params")]
    InvalidParams,

    /// Internal error (-32603)
    #[fail(display = "Internal error")]
    InternalError,

    /// Server error (-32000)
    #[fail(display = "Server error")]
    ServerError,

    /// Other error types
    #[fail(display = "Error (code: {})", 0)]
    Other(i32),
}

impl Code {
    /// Get the integer error value for this code
    pub fn value(self) -> i32 {
        i32::from(self)
    }
}

impl From<i32> for Code {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::HttpError,
            -32700 => Self::ParseError,
            -32600 => Self::InvalidRequest,
            -32601 => Self::MethodNotFound,
            -32602 => Self::InvalidParams,
            -32603 => Self::InternalError,
            -32000 => Self::ServerError,
            other => Self::Other(other),
        }
    }
}

impl From<Code> for i32 {
    fn from(code: Code) -> Self {
        match code {
            Code::HttpError => 0,
            Code::ParseError => -32700,
            Code::InvalidRequest => -32600,
            Code::MethodNotFound => -32601,
            Code::InvalidParams => -32602,
            Code::InternalError => -32603,
            Code::ServerError => -32000,
            Code::Other(other) => other,
        }
    }
}

impl<'de> Deserialize<'de> for Code {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::from(i32::deserialize(deserializer)?))
    }
}

impl Serialize for Code {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.value().serialize(serializer)
    }
}
