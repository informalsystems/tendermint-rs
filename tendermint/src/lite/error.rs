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

    #[error("Request failed")]
    RequestFailed,

    #[error("A valid threshold is `1/3 <= threshold <= 1`, got: {got}")]
    InvalidTrustThreshold { got: String },

    #[error("Implementation specific error")]
    ImplementationSpecific,
}

impl Kind {
    /// Add additional context.
    pub fn context(self, source: impl Into<BoxError>) -> Context<Kind> {
        Context::new(self, Some(source.into()))
    }
}

// TODO test
//  Err(Kind::ImplementationSpecific
//                    .context("validator set is empty, or, invalid hash algo".to_string())
//                    .into()))
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
