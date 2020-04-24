use anomaly::{BoxError, Context};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
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

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum ErrorKind {
    #[error("header from the future: header_time={header_time:?} now={now:?}")]
    HeaderFromTheFuture {
        header_time: SystemTime,
        now: SystemTime,
    },
    #[error("implementation specific")]
    ImplementationSpecific,
    #[error(
        "insufficient validators overlap: total_power={total_power} signed_power={signed_power}"
    )]
    InsufficientValidatorsOverlap { total_power: u64, signed_power: u64 },
    #[error("insufficient voting power: total_power={total_power} voting_power={voting_power}")]
    InsufficientVotingPower { total_power: u64, voting_power: u64 },
    #[error("invalid commit: total_power={total_power} signed_power={signed_power}")]
    InvalidCommit { total_power: u64, signed_power: u64 },
    #[error("invalid commit value: header_hash={header_hash} commit_hash={commit_hash}")]
    InvalidCommitValue {
        header_hash: Hash,
        commit_hash: Hash,
    },
    #[error("invalid next validator set: header_next_validators_hash={header_next_validators_hash} next_validators_hash={next_validators_hash}")]
    InvalidNextValidatorSet {
        header_next_validators_hash: Hash,
        next_validators_hash: Hash,
    },
    #[error("invalid validator set: header_validators_hash={header_validators_hash} validators_hash={validators_hash}")]
    InvalidValidatorSet {
        header_validators_hash: Hash,
        validators_hash: Hash,
    },
    #[error("non increasing height: got={got} expected={expected}")]
    NonIncreasingHeight { got: Height, expected: Height },
    #[error("non monotonic BFT time: header_bft_time={header_bft_time:?} trusted_header_bft_time={trusted_header_bft_time:?}")]
    NonMonotonicBftTime {
        header_bft_time: SystemTime,
        trusted_header_bft_time: SystemTime,
    },
    #[error("not withing trust period: at={at:?} now={now:?}")]
    NotWithinTrustPeriod { at: SystemTime, now: SystemTime },
}

impl ErrorKind {
    /// Add additional context (i.e. include a source error and capture a backtrace).
    /// You can convert the resulting `Context` into an `Error` by calling `.into()`.
    pub fn context(self, source: impl Into<BoxError>) -> Context<ErrorKind> {
        Context::new(self, Some(source.into()))
    }
}
