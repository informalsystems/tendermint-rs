//! Error types

use std::net;

use anomaly::{BoxError, Context};

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
    #[error("i/o error")]
    Io,
}

impl Kind {
    /// Add additional context (i.e. include a source error and capture a backtrace).
    /// You can convert the resulting `Context` into an `Error` by calling `.into()`.
    pub fn context(self, source: impl Into<BoxError>) -> Context<Self> {
        Context::new(self, Some(source.into()))
    }
}
