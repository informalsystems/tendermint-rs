use anomaly::{BoxError, Context};
use thiserror::Error;

use crate::prelude::*;

/// Ensure a condition holds, returning an error if it doesn't (ala `assert`)
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $kind:expr) => {
        if !($cond) {
            return Err($kind.into());
        }
    };
}

pub type Error = anomaly::Error<ErrorKind>;

#[derive(Debug, Clone, Error)]
pub enum ErrorKind {
    #[error("verifier error")]
    Verifier(#[source] VerifierError),
}

impl ErrorKind {
    /// Add additional context (i.e. include a source error and capture a backtrace).
    /// You can convert the resulting `Context` into an `Error` by calling `.into()`.
    pub fn context(self, source: impl Into<BoxError>) -> Context<ErrorKind> {
        Context::new(self, Some(source.into()))
    }
}
