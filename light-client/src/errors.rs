//! Toplevel errors raised by the light client.

use std::fmt::Debug;

use anomaly::{BoxError, Context};
use crossbeam_channel as crossbeam;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    components::io::IoError,
    light_client::Options,
    predicates::errors::VerificationError,
    types::{Height, PeerId, Status},
};

pub type Error = anomaly::Error<ErrorKind>;

#[derive(Debug, Clone, Error, PartialEq, Serialize, Deserialize)]
pub enum ErrorKind {
    #[error("I/O error: {0}")]
    Io(#[from] IoError),

    #[error("store error")]
    Store,

    #[error("no primary")]
    NoPrimary,

    #[error("no witnesses")]
    NoWitnesses,

    #[error("no witness left")]
    NoWitnessLeft,

    #[error("fork detected peers={0:?}")]
    ForkDetected(Vec<PeerId>),

    #[error("no initial trusted state")]
    NoInitialTrustedState,

    #[error("no trusted state")]
    NoTrustedState(Status),

    #[error("target height ({target_height}) is lower than trusted state ({trusted_height})")]
    TargetLowerThanTrustedState {
        target_height: Height,
        trusted_height: Height,
    },

    #[error("trusted state outside of trusting period")]
    TrustedStateOutsideTrustingPeriod {
        trusted_height: Height,
        options: Options,
    },

    #[error("bisection for target at height {0} failed when reached trusted state at height {1}")]
    BisectionFailed(Height, Height),

    #[error("invalid light block: {0}")]
    InvalidLightBlock(#[source] VerificationError),

    #[error("internal channel disconnected")]
    ChannelDisconnected,
}

impl ErrorKind {
    /// Add additional context (i.e. include a source error and capture a backtrace).
    /// You can convert the resulting `Context` into an `Error` by calling `.into()`.
    pub fn context(self, source: impl Into<BoxError>) -> Context<Self> {
        Context::new(self, Some(source.into()))
    }
}

pub trait ErrorExt {
    /// Whether this error means that the light block
    /// cannot be trusted w.r.t. the latest trusted state.
    fn not_enough_trust(&self) -> bool;

    /// Whether this error means that the light block has expired,
    /// ie. it's outside of the trusting period.
    fn has_expired(&self) -> bool;

    /// Whether this error means that a timeout occured when
    /// querying a node.
    fn is_timeout(&self) -> bool;
}

impl ErrorExt for ErrorKind {
    fn not_enough_trust(&self) -> bool {
        if let Self::InvalidLightBlock(e) = self {
            e.not_enough_trust()
        } else {
            false
        }
    }

    fn has_expired(&self) -> bool {
        if let Self::InvalidLightBlock(e) = self {
            e.has_expired()
        } else {
            false
        }
    }

    /// Whether this error means that a timeout occured when querying a node.
    fn is_timeout(&self) -> bool {
        if let Self::Io(e) = self {
            e.is_timeout()
        } else {
            false
        }
    }
}

impl<T: Debug + Send + Sync + 'static> From<crossbeam::SendError<T>> for ErrorKind {
    fn from(_err: crossbeam::SendError<T>) -> Self {
        Self::ChannelDisconnected
    }
}

impl From<crossbeam::RecvError> for ErrorKind {
    fn from(_err: crossbeam::RecvError) -> Self {
        Self::ChannelDisconnected
    }
}
