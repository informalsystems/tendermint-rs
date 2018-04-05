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
}

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Self {
        err!(IoError, "{}", other)
    }
}
