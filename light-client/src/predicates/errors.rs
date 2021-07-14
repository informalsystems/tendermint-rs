//! Errors which may be raised when verifying a `LightBlock`

use flex_error::define_error;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::errors::ErrorExt;
use crate::operations::voting_power::VotingPowerTally;
use crate::types::{Hash, Height, Time, Validator, ValidatorAddress};
use tendermint::account::Id;

define_error! {
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    VerificationError {
        HeaderFromTheFuture
            {
                header_time: Time,
                now: Time,
            }
            | e | {
                format_args!("header from the future: header_time={0} now={1}",
                    e.header_time, e.now)
            },

        ImplementationSpecific
            {
                detail: String,
            }
            | e | {
                format_args!("implementation specific: {0}",
                    e.detail)
            },

        NotEnoughTrust
            {
                tally: VotingPowerTally,
            }
            | e | {
                format_args!("not enough trust because insufficient validators overlap: {0}",
                    e.tally)
            },

        InsufficientSignersOverlap
            {
                tally: VotingPowerTally,
            }
            | e | {
                format_args!("insufficient signers overlap: {0}",
                    e.tally)
            },

        DuplicateValidator
            {
                address: ValidatorAddress,
            }
            | e | {
                format_args!("duplicate validator with address {0}",
                    e.address)
            },

        InvalidSignature
            {
                signature: Vec<u8>,
                validator: Box<Validator>,
                sign_bytes: Vec<u8>,
            }
            | e | {
                format_args!("Couldn't verify signature `{:?}` with validator `{:?}` on sign_bytes `{:?}`",
                    e.signature, e.validator, e.sign_bytes)
            },

        InvalidCommitValue
            {
                header_hash: Hash,
                commit_hash: Hash,
            }
            | e | {
                format_args!("invalid commit value: header_hash={0} commit_hash={1}",
                    e.header_hash, e.commit_hash)
            },

        InvalidNextValidatorSet
            {
                header_next_validators_hash: Hash,
                next_validators_hash: Hash,
            }
            | e | {
                format_args!("invalid next validator set: header_next_validators_hash={0} next_validators_hash={1}",
                    e.header_next_validators_hash, e.next_validators_hash)
            },

        InvalidValidatorSet
            {
                header_validators_hash: Hash,
                validators_hash: Hash,
            }
            | e | {
                format_args!("invalid validator set: header_validators_hash={0} validators_hash={1}",
                    e.header_validators_hash, e.validators_hash)
            },

        NonIncreasingHeight
            {
                got: Height,
                expected: Height,
            }
            | e | {
                format_args!("non increasing height: got={0} expected={1}",
                    e.got, e.expected)
            },

        NonMonotonicBftTime
            {
                header_bft_time: Time,
                trusted_header_bft_time: Time,
            }
            | e | {
                format_args!("non monotonic BFT time: header_bft_time={0} trusted_header_bft_time={1}",
                    e.header_bft_time, e.trusted_header_bft_time)
            },

        NotWithinTrustPeriod
            {
                expires_at: Time,
                now: Time,
            }
            | e | {
                format_args!("not withing trusting period: expires_at={0} now={1}",
                    e.expires_at, e.now)
            },

        NoSignatureForCommit
            | _ | { "no signatures for commit"  },

        MismatchPreCommitLength
            {
                pre_commit_length: usize,
                validator_length: usize,
            }
            | e | {
                format_args!(
                    "pre-commit length: {} doesn't match validator length: {}",
                    e.pre_commit_length,
                    e.validator_length
                )
            },

        FaultySigner
            {
                signer: Id,
                validator_set: Hash
            }
            | e | {
                format_args!(
                    "Found a faulty signer ({}) not present in the validator set ({})",
                    e.signer,
                    e.validator_set
                )
            },

    }
}

impl ErrorExt for VerificationErrorDetail {
    fn not_enough_trust(&self) -> Option<VotingPowerTally> {
        match &self {
            Self::NotEnoughTrust(e) => Some(e.tally),
            _ => None,
        }
    }

    fn has_expired(&self) -> bool {
        matches!(self, Self::NotWithinTrustPeriod { .. })
    }

    fn is_timeout(&self) -> Option<Duration> {
        None
    }
}
