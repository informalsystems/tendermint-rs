use anomaly::{BoxError, Context};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::prelude::*;

pub type Error = anomaly::Error<ErrorKind>;

#[derive(Debug, Clone, Error, PartialEq, Serialize, Deserialize)]
pub enum ErrorKind {
    #[error("I/O error: {0}")]
    Io(#[from] IoError),

    #[error("store error")]
    Store,

    #[error("no initial trusted state")]
    NoInitialTrustedState,

    #[error("latest trusted state outside of trusting period")]
    TrustedStateOutsideTrustingPeriod {
        trusted_state: Box<TrustedState>,
        options: Options,
    },

    #[error("bisection for target at height {0} failed when reached trusted state at height {1}")]
    BisectionFailed(Height, Height),

    #[error("invalid light block: {0}")]
    InvalidLightBlock(#[source] VerificationError),
}

impl ErrorKind {
    /// Add additional context (i.e. include a source error and capture a backtrace).
    /// You can convert the resulting `Context` into an `Error` by calling `.into()`.
    pub fn context(self, source: impl Into<BoxError>) -> Context<Self> {
        Context::new(self, Some(source.into()))
    }
}
