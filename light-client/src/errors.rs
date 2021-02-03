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
    types::{Hash, Height, LightBlock, PeerId, Status},
};

/// An error raised by this library
pub type Error = anomaly::Error<ErrorKind>;

/// The various error kinds raised by this library
#[derive(Debug, Clone, Error, PartialEq, Serialize, Deserialize)]
pub enum ErrorKind {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] IoError),

    /// Store error
    #[error("store error")]
    Store,

    /// No primary
    #[error("no primary")]
    NoPrimary,

    /// No witnesses
    #[error("no witnesses")]
    NoWitnesses,

    /// No witness left
    #[error("no witness left")]
    NoWitnessLeft,

    /// A fork has been detected between some peers
    #[error("fork detected peers={0:?}")]
    ForkDetected(Vec<PeerId>),

    /// No initial trusted state
    #[error("no initial trusted state")]
    NoInitialTrustedState,

    /// No trusted state
    #[error("no trusted state")]
    NoTrustedState(Status),

    /// Target height for the light client lower than latest trusted state height
    #[error("target height ({target_height}) is lower than trusted state ({trusted_height})")]
    TargetLowerThanTrustedState {
        /// Target height
        target_height: Height,
        /// Latest trusted state height
        trusted_height: Height,
    },

    /// The trusted state is outside of the trusting period
    #[error("trusted state outside of trusting period")]
    TrustedStateOutsideTrustingPeriod {
        /// Trusted state
        trusted_state: Box<LightBlock>,
        /// Light client options
        options: Options,
    },

    /// Bisection failed when reached trusted state
    #[error("bisection for target at height {0} failed when reached trusted state at height {1}")]
    BisectionFailed(Height, Height),

    /// Verification failed for a light block
    #[error("invalid light block: {0}")]
    InvalidLightBlock(#[source] VerificationError),

    /// Hash mismatch between two adjacent headers
    #[error("hash mismatch between two adjacent headers: {h1} != {h2}")]
    InvalidAdjacentHeaders {
        /// Hash #1
        h1: Hash,
        /// Hash #2
        h2: Hash,
    },

    /// Missing last_block_id field for header at given height
    #[error("missing last_block_id for header at height {0}")]
    MissingLastBlockId(Height),

    /// Internal channel disconnected
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

/// Extension methods for `ErrorKind`
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
