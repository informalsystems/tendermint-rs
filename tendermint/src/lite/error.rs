//! All error types tied to the light client.

use crate::Hash;
use anomaly::{BoxError, Context};
use std::time::{SystemTime, SystemTimeError};
use thiserror::Error;

/// The main error type verification methods will return.
/// See [`ErrorKind`] for the different kind of errors.
pub type Error = anomaly::Error<Kind>;

#[derive(Clone, Debug, Error)]
pub enum Kind {
    #[error("old header has expired at {at:?} (now: {now:?})")]
    Expired { at: SystemTime, now: SystemTime },

    #[error("duration error {:?}", _0)]
    DurationOutOfRange(#[from] SystemTimeError),

    #[error("expected height >= {expected} (got: {got})")]
    NonIncreasingHeight { got: u64, expected: u64 },

    #[error("header's validator hash does not match actual validator hash ({header_val_hash:?}!={val_hash:?})")]
    InvalidValidatorSet {
        header_val_hash: Hash,
        val_hash: Hash,
    },

    #[error("header's next validator hash does not match next_val_hash ({header_next_val_hash:?}!={next_val_hash:?})")]
    InvalidNextValidatorSet {
        header_next_val_hash: Hash,
        next_val_hash: Hash,
    },

    #[error(
        "header hash does not match the hash in the commit ({header_hash:?}!={commit_hash:?})"
    )]
    InvalidCommitValue {
        header_hash: Hash,
        commit_hash: Hash,
    }, // commit is not for the header we expected

    #[error("signed voting power ({signed}) do not account for +2/3 of the total voting power: ({total})")]
    InvalidCommit { total: u64, signed: u64 },

    #[error("signed voting power ({}) is too small fraction of total voting power: ({}), threshold: {}",
        .signed, .total, .trust_treshold
    )]
    InsufficientVotingPower {
        total: u64,
        signed: u64,
        trust_treshold: String,
    }, // trust threshold (default +1/3) is not met

    #[error("{0:?}")]
    RequestFailed(String), // Note: we don't want to depend on the tendermint rpc error types, otherwise we should do #[from] Error instead of expecting a string

    #[error("A valid threshold is `1/3 <= threshold <= 1`, got: {got}")]
    InvalidTrustThreshold { got: String },

    #[error("Implementation specific error: {0}")]
    ImplementationSpecific(String),
}

impl Kind {
    /// Add additional context.
    pub fn context(self, source: impl Into<BoxError>) -> Context<Kind> {
        Context::new(self, Some(source.into()))
    }
}
