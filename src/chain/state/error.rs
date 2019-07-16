use abscissa_core::Error;
use std::fmt::{self, Display};

/// Error type
#[derive(Debug)]
pub struct StateError(pub(crate) Error<StateErrorKind>);

/// Kinds of errors
#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum StateErrorKind {
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

impl StateError {
    /// Get the kind of error
    pub fn kind(&self) -> StateErrorKind {
        *self.0.kind()
    }
}

impl From<Error<StateErrorKind>> for StateError {
    fn from(other: Error<StateErrorKind>) -> Self {
        StateError(other)
    }
}

impl Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
