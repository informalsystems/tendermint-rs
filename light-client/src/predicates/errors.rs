//! Errors which may be raised when verifying a `LightBlock`

use anomaly::{BoxError, Context};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::errors::ErrorExt;
use crate::operations::voting_power::VotingPowerTally;
use crate::types::{Hash, Height, Time, Validator, ValidatorAddress};

/// The various errors which can be raised by the verifier component,
/// when validating or verifying a light block.
#[derive(Debug, Clone, Error, PartialEq, Serialize, Deserialize)]
pub enum VerificationError {
    /// The header is from the future
    #[error("header from the future: header_time={header_time} now={now}")]
    HeaderFromTheFuture {
        /// Time in the header
        header_time: Time,
        /// Current time
        now: Time,
    },

    /// Implementation specific error, for the purpose of extensibility
    #[error("implementation specific: {0}")]
    ImplementationSpecific(String),

    /// Not enough trust because insufficient validators overlap
    #[error("not enough trust because insufficient validators overlap: {0}")]
    NotEnoughTrust(VotingPowerTally),

    /// Insufficient signers overlap
    #[error("insufficient signers overlap: {0}")]
    InsufficientSignersOverlap(VotingPowerTally),

    /// Duplicate validator in commit signatures
    #[error("duplicate validator with address {0}")]
    DuplicateValidator(ValidatorAddress),

    /// Invalid commit signature
    #[error("Couldn't verify signature `{signature:?}` with validator `{validator:?}` on sign_bytes `{sign_bytes:?}`")]
    InvalidSignature {
        /// Signature as a byte array
        signature: Vec<u8>,
        /// Validator which provided the signature
        validator: Box<Validator>,
        /// Bytes which were signed
        sign_bytes: Vec<u8>,
    },

    /// Invalid commit
    #[error("invalid commit value: header_hash={header_hash} commit_hash={commit_hash}")]
    InvalidCommitValue {
        /// Header hash
        header_hash: Hash,
        /// Commit hash
        commit_hash: Hash,
    },

    /// Hash mismatch for the next validator set
    #[error("invalid next validator set: header_next_validators_hash={header_next_validators_hash} next_validators_hash={next_validators_hash}")]
    InvalidNextValidatorSet {
        /// Next validator set hash
        header_next_validators_hash: Hash,
        /// Validator set hash
        next_validators_hash: Hash,
    },

    /// Hash mismatch for the validator set
    #[error("invalid validator set: header_validators_hash={header_validators_hash} validators_hash={validators_hash}")]
    InvalidValidatorSet {
        /// Hash of validator set stored in header
        header_validators_hash: Hash,
        /// Actual hash of validator set in header
        validators_hash: Hash,
    },

    /// Unexpected header of non-increasing height compared to what was expected
    #[error("non increasing height: got={got} expected={expected}")]
    NonIncreasingHeight {
        /// Actual height of header
        got: Height,
        /// Expected minimum height
        expected: Height,
    },

    /// BFT Time between the trusted state and a header does not increase monotonically
    #[error("non monotonic BFT time: header_bft_time={header_bft_time} trusted_header_bft_time={trusted_header_bft_time}")]
    NonMonotonicBftTime {
        /// BFT time of the untrusted header
        header_bft_time: Time,
        /// BFT time of the trusted header
        trusted_header_bft_time: Time,
    },

    /// Trusted state not within the trusting period
    #[error("not withing trusting period: expires_at={expires_at} now={now}")]
    NotWithinTrustPeriod {
        /// Expiration time of the header
        expires_at: Time,
        /// Current time
        now: Time,
    },
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
        matches!(self, Self::NotEnoughTrust { .. })
    }

    fn has_expired(&self) -> bool {
        matches!(self, Self::NotWithinTrustPeriod { .. })
    }

    fn is_timeout(&self) -> bool {
        false
    }
}
