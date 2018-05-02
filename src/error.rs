//! Error types

use std::io;

/// Error type
#[derive(Clone, Eq, PartialEq, Debug, Fail)]
pub enum Error {
    /// Error in configuration file
    #[fail(display = "{}", description)]
    ConfigError {
        /// Description of the error
        description: String,
    },

    /// Malformatted or otherwise invalid cryptographic key
    #[fail(display = "{}", description)]
    InvalidKey {
        /// Description of the error
        description: String,
    },

    /// Input/output error
    #[fail(display = "{}", description)]
    IoError {
        /// Description of the error
        description: String,
    },

    /// Signing operation failed
    #[fail(display = "{}", description)]
    SigningError {
        /// Description of the error
        description: String,
    },
}

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Self {
        Error::IoError {
            description: other.to_string(),
        }
    }
}

/// Create a new error (of a given enum variant) with a formatted message
macro_rules! err {
    ($variant:ident, $msg:expr) => {
        ::error::Error::$variant { description: $msg.to_owned() }
    };
    ($variant:ident, $fmt:expr, $($arg:tt)+) => {
        ::error::Error::$variant { description: format!($fmt, $($arg)+) }
    };
}
