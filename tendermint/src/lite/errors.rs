//! All error types tied to the light client.

use crate::Hash;
use failure::*;
use std::time::{SystemTime, SystemTimeError};

#[derive(Clone, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "old header has expired at {:?} (now: {:?})", at, now)]
    Expired { at: SystemTime, now: SystemTime },

    #[fail(display = "duration error {:?}", _0)]
    DurationOutOfRange(#[cause] SystemTimeError),

    #[fail(display = "expected height >= {} (got: {})", expected, got)]
    NonIncreasingHeight { got: u64, expected: u64 },

    #[fail(display = "could not verify signature")]
    InvalidSignature, // TODO: deduplicate with tendermint::ErrorKind::SignatureInvalid

    #[fail(
        display = "header's validator hash does not match actual validator hash ({:?}!={:?})",
        header_val_hash, val_hash
    )]
    InvalidValidatorSet {
        header_val_hash: Hash,
        val_hash: Hash,
    },

    #[fail(
        display = "header's next validator hash does not match next_val_hash ({:?}!={:?})",
        header_next_val_hash, next_val_hash
    )]
    InvalidNextValidatorSet {
        header_next_val_hash: Hash,
        next_val_hash: Hash,
    },

    #[fail(
        display = "header hash does not match the hash in the commit ({:?}!={:?})",
        header_hash, commit_hash
    )]
    InvalidCommitValue {
        header_hash: Hash,
        commit_hash: Hash,
    }, // commit is not for the header we expected

    #[fail(display = "error validating commit signatures: {}", info)]
    InvalidCommitSignatures { info: String }, // Note: this is only used by implementation (ie. expected return in Commit::validate())

    #[fail(
        display = "signed voting power ({}) do not account for +2/3 of the total voting power: ({})",
        signed, total
    )]
    InvalidCommit { total: u64, signed: u64 },

    #[fail(
        display = "signed voting power ({}) is too small fraction of total voting power: ({}), threshold: {}",
        signed, total, trust_treshold
    )]
    InsufficientVotingPower {
        total: u64,
        signed: u64,
        trust_treshold: String,
    }, // trust threshold (default +1/3) is not met

    #[fail(display = "{:?}", _0)]
    RequestFailed(String),

    #[fail(display = "A valid threshold is `1/3 <= threshold <= 1`, got: {}", got)]
    InvalidTrustThreshold { got: String },
}
