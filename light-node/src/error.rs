//! Error types

use std::io;
use std::net;

/// Error type
pub type Error = anomaly::Error<Kind>;

/// Kinds of errors
#[derive(Clone, Debug, thiserror::Error)]
pub enum Kind {
    /// Error when parsing an address.
    #[error(transparent)]
    AddrParse(#[from] net::AddrParseError),

    /// Error in configuration file
    #[error("config error")]
    Config,

    /// Input/output error
    #[error("i/o error: {0}")]
    Io(String),
}

impl From<io::Error> for Kind {
    fn from(err: io::Error) -> Self {
        Self::Io(format!("{}", err))
    }
}
