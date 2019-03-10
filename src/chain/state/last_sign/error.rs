use abscissa::Error;
use std::fmt::{self, Display};

/// Error type
#[derive(Debug)]
pub struct LastSignError(pub(crate) Error<LastSignErrorKind>);

/// Kinds of errors
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum LastSignErrorKind {
    /// Height regressed
    #[fail(display = "height regression")]
    HeightRegression,

    /// Step regressed
    #[fail(display = "step regression")]
    StepRegression,

    /// Round regressed
    #[fail(display = "round regression")]
    RoundRegression,

    /// Double sign detected
    #[fail(display = "double sign detected")]
    DoubleSign,

    /// Error syncing state to disk
    #[fail(display = "error syncing state to disk")]
    SyncError,
}

impl From<Error<LastSignErrorKind>> for LastSignError {
    fn from(other: Error<LastSignErrorKind>) -> Self {
        LastSignError(other)
    }
}

impl Display for LastSignError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
