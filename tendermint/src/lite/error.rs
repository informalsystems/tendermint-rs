//! All error types tied to the light client.

use crate::Hash;
use anomaly::{BoxError, Context};
use std::time::{SystemTime, SystemTimeError};
use thiserror::Error;

/// The main error type verification methods will return.
/// See [`Kind`] for the different kind of errors.
pub type Error = anomaly::Error<Kind>;

/// All error kinds related to the light client.
#[derive(Clone, Debug, Error)]
pub enum Kind {
    /// The provided header expired.
    #[error("old header has expired at {at:?} (now: {now:?})")]
    Expired { at: SystemTime, now: SystemTime },

    /// Trusted header is from the future.
    #[error("duration error {:?}", _0)]
    DurationOutOfRange(#[from] SystemTimeError),

    /// Header height smaller than expected.
    #[error("expected height >= {expected} (got: {got})")]
    NonIncreasingHeight { got: u64, expected: u64 },

    /// Header time is in the past compared to already trusted header.
    #[error("untrusted header time <= trusted header time")]
    NonIncreasingTime,

    /// Invalid validator hash.
    #[error("header's validator hash does not match actual validator hash ({header_val_hash:?}!={val_hash:?})")]
    InvalidValidatorSet {
        header_val_hash: Hash,
        val_hash: Hash,
    },

    /// Invalid next validator hash.
    #[error("header's next validator hash does not match next_val_hash ({header_next_val_hash:?}!={next_val_hash:?})")]
    InvalidNextValidatorSet {
        header_next_val_hash: Hash,
        next_val_hash: Hash,
    },

    /// Commit is not for the header we expected.
    #[error(
        "header hash does not match the hash in the commit ({header_hash:?}!={commit_hash:?})"
    )]
    InvalidCommitValue {
        header_hash: Hash,
        commit_hash: Hash,
    },

    /// Signed power does not account for +2/3 of total voting power.
    #[error("signed voting power ({signed}) do not account for +2/3 of the total voting power: ({total})")]
    InvalidCommit { total: u64, signed: u64 },

    /// This means the trust threshold (default +1/3) is not met.
    #[error("signed voting power ({}) is too small fraction of total voting power: ({}), threshold: {}",
        .signed, .total, .trust_treshold
    )]
    InsufficientVotingPower {
        total: u64,
        signed: u64,
        trust_treshold: String,
    },

    /// This is returned if an invalid TrustThreshold is created.
    #[error("A valid threshold is `1/3 <= threshold <= 1`, got: {got}")]
    InvalidTrustThreshold { got: String },

    /// Use the [`Kind::context`] method to wrap the underlying error of
    /// the implementation, if any.
    #[error("Request failed")]
    RequestFailed,

    /// Use the [`Kind::context`] method to wrap the underlying error of
    /// the implementation, if any.
    #[error("Implementation specific error")]
    ImplementationSpecific,
}

impl Kind {
    /// Add additional context.
    pub fn context(self, source: impl Into<BoxError>) -> Context<Kind> {
        Context::new(self, Some(source.into()))
    }
}

#[cfg(test)]
mod tests {
    use crate::lite::error::{Error as LiteErr, Kind};
    use std::error::Error;

    #[test]
    fn test_implementation_specific() {
        let e: Result<(), LiteErr> = Err(Kind::ImplementationSpecific
            .context("Some implementation specific error which doesn't need to be a string")
            .into());
        let err_kind = e.unwrap_err();
        // could we match against the Kind?
        assert_eq!(
            format!("{:?}", &Kind::ImplementationSpecific),
            format!("{:?}", err_kind.kind())
        );
        // do we still have that context error?
        assert_eq!(
            format!(
                "{:?}",
                Some("Some implementation specific error which doesn't need to be a string")
            ),
            format!("{:?}", err_kind.source())
        );

        // Can we do the same with some actual implementation of std::error::Error?
        // produce sth we know yields an Error (we don't care what it is):
        let res = "xc".parse::<u32>();
        let source_err = res.unwrap_err();
        let e: Result<(), LiteErr> = Err(Kind::ImplementationSpecific
            .context(source_err.clone())
            .into());
        let err_kind = e.unwrap_err();
        assert_eq!(
            format!("{:?}", &Kind::ImplementationSpecific),
            format!("{:?}", err_kind.kind())
        );
        assert_eq!(
            format!("{}", source_err),
            format!("{}", err_kind.source().unwrap())
        );
    }
}
