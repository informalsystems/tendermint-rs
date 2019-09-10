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
    pub fn new(code: Code, data: Option<String>) -> Error {
        let message = code.to_string();
        Error {
            code,
            message,
            data,
        }
    }

    /// Create a new invalid parameter error
    pub fn invalid_params(data: &str) -> Error {
        Error::new(Code::InvalidParams, Some(data.to_string()))
    }

    /// Create a new method-not-found error
    pub fn method_not_found(name: &str) -> Error {
        Error::new(Code::MethodNotFound, Some(name.to_string()))
    }

    /// Create a new parse error
    pub fn parse_error<E>(error: E) -> Error
    where
        E: Display,
    {
        Error::new(Code::ParseError, Some(error.to_string()))
    }

    /// Create a new server error
    pub fn server_error<D>(data: D) -> Error
    where
        D: Display,
    {
        Error::new(Code::ServerError, Some(data.to_string()))
    }

    /// Obtain the `rpc::error::Code` for this error
    pub fn code(&self) -> Code {
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

impl From<hyper::Error> for Error {
    fn from(hyper_error: hyper::Error) -> Error {
        panic!("what am I supposed to do with this? {:?}", hyper_error);
    }
}

/// Tendermint RPC error codes.
///
/// See `func RPC*Error()` definitions in:
/// <https://github.com/tendermint/tendermint/blob/master/rpc/lib/types/types.go>
#[derive(Copy, Clone, Debug, Eq, Fail, Hash, PartialEq, PartialOrd, Ord)]
pub enum Code {
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
    fn from(value: i32) -> Code {
        match value {
            -32700 => Code::ParseError,
            -32600 => Code::InvalidRequest,
            -32601 => Code::MethodNotFound,
            -32602 => Code::InvalidParams,
            -32603 => Code::InternalError,
            -32000 => Code::ServerError,
            other => Code::Other(other),
        }
    }
}

impl From<Code> for i32 {
    fn from(code: Code) -> i32 {
        match code {
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
        Ok(Code::from(i32::deserialize(deserializer)?))
    }
}

impl Serialize for Code {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.value().serialize(serializer)
    }
}
