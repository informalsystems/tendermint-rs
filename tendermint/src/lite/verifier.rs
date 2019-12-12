#[allow(clippy::all)]
use crate::lite::{Commit, Error, Header, SignedHeader, TrustThreshold, ValidatorSet};
use crate::Time;
use std::time::Duration;

/// Returns an error if the header has expired according to the given
/// trusting_period and current time. If so, the verifier must be reset subjectively.
/// NOTE: this doesn't belong here. It should be called by something that handles whether to trust
/// a verified commit. Verified here is really just about the header/commit/validators. Time is an
/// external concern :)
pub fn expired<H>(last_header: &H, trusting_period: Duration, now: Time) -> Result<(), Error>
where
    H: Header,
{
    if let Ok(passed) = now.duration_since(last_header.bft_time()) {
        if passed > trusting_period {
            return Err(Error::Expired);
        }
    }
    // TODO move this out of the verifier and deal with overflows etc (proper err handling)
    Ok(())
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

// TODO: documentation!
pub fn verify<S, H, C, V, L>(
    sh1: &S,
    h1_next_vals: &V,
    sh2: &S,
    h2_vals: &V,
    trust_level: L,
) -> Result<(), Error>
where
    S: SignedHeader<H, C>,
    H: Header,
    V: ValidatorSet,
    C: Commit,
    L: TrustThreshold,
{
    let h1 = sh1.header();
    let h2 = sh2.header();
    let commit = sh2.commit();
    if h2.height() == h1.height().increment() {
        if h2.validators_hash() != h1_next_vals.hash() {
            return Err(Error::InvalidNextValidatorSet);
        }
    } else {
        // ensure that +1/3 of last trusted validators signed correctly
        if let Err(e) = verify_commit_trusting(h1_next_vals, commit, trust_level) {
            return Err(e);
        }
    }

    verify_header_and_commit(h2, sh2.commit(), h2_vals)
}

// Validate the validators and commit against the header.
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
fn verify_header_and_commit<H, V, C>(header: &H, commit: &C, validators: &V) -> Result<(), Error>
where
    H: Header,
    V: ValidatorSet,
    C: Commit,
{
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
