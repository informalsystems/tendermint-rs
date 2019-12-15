//! Main verification functions that can be used to implement a light client.
//!
//!
//! # Examples
//!
//! ```
//! // TODO: add a proper example maybe showing how a `can_trust_bisection`
//! // looks using the types and methods in this crate/module.
//!```

use crate::lite::{Commit, Error, Header, SignedHeader, TrustThreshold, ValidatorSet};
use std::time::{Duration, SystemTime};

/// Returns an error if the header has expired according to the given
/// trusting_period and current time. If so, the verifier must be reset subjectively.
/// NOTE: this doesn't belong here. It should be called by something that handles whether to trust
/// a verified commit. Verified here is really just about the header/commit/validators. Time is an
/// external concern :)
pub fn expired<H>(last_header: &H, trusting_period: Duration, now: SystemTime) -> Result<(), Error>
where
    H: Header,
{
    match now.duration_since(last_header.bft_time().into()) {
        Ok(passed) => {
            if passed > trusting_period {
                return Err(Error::Expired);
            }
            Ok(())
        }
        Err(_) => Err(Error::DurationOutOfRange),
    }
}

fn validate_next_vals<H, V>(header: H, next_vals: &V) -> Result<(), Error>
where
    H: Header,
    V: ValidatorSet,
{
    // ensure the next validators in the header matches what was supplied.
    if header.next_validators_hash() != next_vals.hash() {
        return Err(Error::InvalidNextValidatorSet);
    }

    Ok(())
}

/// Captures the skipping condition, i.e., it defines when we can trust the header
/// h2 based on header h1.
/// Note that h1 and h2 have already passed basic validation by calling  `verify`.
/// `Error::InsufficientVotingPower`is returned when there is not enough intersection
/// between validator sets to have skipping condition true.
pub fn check_support<S, V, L>(
    h1: &S,
    h1_next_vals: &V,
    h2: &S,
    trust_threshold: L,
    trusting_period: Duration,
    now: SystemTime,
) -> Result<(), Error>
where
    S: SignedHeader,
    V: ValidatorSet,
    L: TrustThreshold,
{
    if let Err(err) = expired(h1.header(), trusting_period, now) {
        return Err(err);
    }

    if h2.header().height() == h1.header().height().increment()
        && h2.header().validators_hash() != h1_next_vals.hash()
    {
        return Err(Error::InvalidNextValidatorSet);
    }
    verify_commit_trusting(h1_next_vals, h2.commit(), trust_threshold)
}

/// Validate the validators and commit against the header.
fn validate_vals_and_commit<H, V, C>(header: &H, commit: &C, vals: &V) -> Result<(), Error>
where
    H: Header,
    V: ValidatorSet,
    C: Commit,
{
    // ensure the validators in the header matches what we expect from our state.
    if header.validators_hash() != vals.hash() {
        return Err(Error::InvalidValidatorSet);
    }

    // ensure the commit matches the header.
    if header.hash() != commit.header_hash() {
        return Err(Error::InvalidCommitValue);
    }

    Ok(())
}

/// Verify the commit is valid from the given validators for the header.
pub fn verify<SH, V>(signed_header: &SH, validators: &V) -> Result<(), Error>
where
    SH: SignedHeader,
    V: ValidatorSet,
{
    let header = signed_header.header();
    let commit = signed_header.commit();
    if let Err(e) = validate_vals_and_commit(header, commit, validators) {
        return Err(e);
    }

    // ensure that +2/3 validators signed correctly
    verify_commit_full(validators, commit)
}

/// Verify that +2/3 of the correct validator set signed this commit.
/// NOTE: these validators are expected to be the correct validators for the commit.
fn verify_commit_full<V, C>(vals: &V, commit: &C) -> Result<(), Error>
where
    V: ValidatorSet,
    C: Commit,
{
    let total_power = vals.total_power();
    if vals.len() != commit.votes_len() {
        return Err(Error::InvalidCommitLength);
    }

    let signed_power = commit.voting_power_in(vals)?;

    // check the signers account for +2/3 of the voting power
    if signed_power * 3 <= total_power * 2 {
        return Err(Error::InsufficientVotingPower);
    }

    Ok(())
}

/// Verify that +1/3 of the given validator set signed this commit.
/// NOTE the given validators do not necessarily correspond to the validator set for this commit,
/// but there may be some intersection. The trust_level parameter allows clients to require more
/// than +1/3 by implementing the TrustLevel trait accordingly.
pub fn verify_commit_trusting<V, C, L>(
    validators: &V,
    commit: &C,
    trust_level: L,
) -> Result<(), Error>
where
    V: ValidatorSet,
    C: Commit,
    L: TrustThreshold,
{
    let total_power = validators.total_power();
    let signed_power = commit.voting_power_in(validators)?;

    // check the signers account for +1/3 of the voting power (or more if the
    // trust_level requires so)
    if !trust_level.is_enough_power(signed_power, total_power) {
        return Err(Error::InsufficientVotingPower);
    }

    Ok(())
}
