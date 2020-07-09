use anomaly::{BoxError, Context};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::errors::ErrorExt;
use crate::operations::voting_power::VotingPowerTally;
use crate::types::{Hash, Height, TMValidatorAddress, TMValidatorInfo, Time};

/// The various errors which can be raised by the verifier component,
/// when validating or verifying a light block.
#[derive(Debug, Clone, Error, PartialEq, Serialize, Deserialize)]
pub enum VerificationError {
    #[error("header from the future: header_time={header_time} now={now}")]
    HeaderFromTheFuture { header_time: Time, now: Time },

    #[error("implementation specific: {0}")]
    ImplementationSpecific(String),

    #[error("not enough trust because insufficient validators overlap: {0}")]
    NotEnoughTrust(VotingPowerTally),

    #[error("insufficient signers overlap: {0}")]
    InsufficientSignersOverlap(VotingPowerTally),

    // #[error(
    //     "validators and signatures count do no match: {validators_count} != {signatures_count}"
    // )]
    // ValidatorsAndSignaturesCountMismatch {
    //     validators_count: usize,
    //     signatures_count: usize,
    // },
    #[error("duplicate validator with address {0}")]
    DuplicateValidator(TMValidatorAddress),

    #[error("Couldn't verify signature `{signature:?}` with validator `{validator:?}` on sign_bytes `{sign_bytes:?}`")]
    InvalidSignature {
        signature: Vec<u8>,
        validator: TMValidatorInfo,
        sign_bytes: Vec<u8>,
    },

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

    #[error("non monotonic BFT time: header_bft_time={header_bft_time} trusted_header_bft_time={trusted_header_bft_time}")]
    NonMonotonicBftTime {
        header_bft_time: Time,
        trusted_header_bft_time: Time,
    },

    #[error("not withing trust period: at={at} now={now}")]
    NotWithinTrustPeriod { at: Time, now: Time },
}

impl VerificationError {
    /// Add additional context (i.e. include a source error and capture a backtrace).
    /// You can convert the resulting `Context` into an `Error` by calling `.into()`.
    pub fn context(self, source: impl Into<BoxError>) -> Context<Self> {
        Context::new(self, Some(source.into()))
    }
}

impl ErrorExt for VerificationError {
    fn not_enough_trust(&self) -> bool {
        if let Self::NotEnoughTrust { .. } = self {
            true
        } else {
            false
        }
    }

    fn has_expired(&self) -> bool {
        if let Self::NotWithinTrustPeriod { .. } = self {
            true
        } else {
            false
        }
    }

    fn is_timeout(&self) -> bool {
        false
    }
}
