#[allow(clippy::all)]
use crate::lite::{
    Commit, Error, Header, SignedHeader, TrustLevel, Validator, ValidatorSet, ValidatorSetLookup,
    Vote,
};
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

pub fn verify<S, H, C, V, L>(
    sh1: S,
    h1_next_vals: V,
    sh2: S,
    h2_vals: V,
    trust_level: L,
) -> Result<(), Error>
where
    S: SignedHeader<H, C>,
    H: Header,
    V: ValidatorSetLookup,
    C: Commit,
    L: TrustLevel,
{
    let h1 = sh1.header();
    let h2 = sh2.header();
    let commit = &sh2.commit();
    if h2.height() == h1.height().increment() {
        if h2.validators_hash() != h1_next_vals.hash() {
            return Err(Error::InvalidNextValidatorSet);
        }
    } else {
        // ensure that +1/3 of last trusted validators signed correctly
        if let Err(e) = verify_commit_trusting(&h1_next_vals, commit, trust_level) {
            return Err(e);
        }
    }
    verify_commit_full(&h2_vals, commit)
}

// Validate the validators and commit against the header.
fn validate_vals_and_commit<H, V, C>(header: H, commit: &C, vals: &V) -> Result<(), Error>
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
fn verify_header_and_commit<H, V, C>(header: H, commit: C, validators: V) -> Result<(), Error>
where
    H: Header,
    V: ValidatorSet,
    C: Commit,
{
    if let Err(e) = validate_vals_and_commit(header, &commit, &validators) {
        return Err(e);
    }

    // ensure that +2/3 validators signed correctly
    verify_commit_full(&validators, &commit)
}

/// Verify the commit is trusted according to the last validators and is valid
/// from the current validators for the header.
pub fn verify_trusting<H, V, C, L>(
    header: H,
    commit: C,
    last_validators: V,
    validators: V,
    trust_level: L,
) -> Result<(), Error>
where
    H: Header,
    V: ValidatorSetLookup,
    C: Commit,
    L: TrustLevel,
{
    // NOTE it might be more prudent to do the cheap validations first
    // before we even call verify_commit_trusting, but not doing that
    // makes the code cleaner and allows us to just call verify directly.

    // ensure that +1/3 of last trusted validators signed correctly
    if let Err(e) = verify_commit_trusting(&last_validators, &commit, trust_level) {
        return Err(e);
    }

    // perform same verification as in sequential case
    verify_header_and_commit(header, commit, validators)
}

/// Verify that +2/3 of the correct validator set signed this commit.
/// NOTE: these validators are expected to be the correct validators for the commit.
fn verify_commit_full<V, C>(vals: &V, commit: &C) -> Result<(), Error>
where
    V: ValidatorSet,
    C: Commit,
{
    let total_power = vals.total_power();
    let mut signed_power: u64 = 0;

    let vals_vec = vals.into_vec();
    let commit_vec = commit.into_vec();

    if vals_vec.len() != commit_vec.len() {
        return Err(Error::InvalidCommitLength);
    }

    // The vals and commit have a 1-to-1 correspondence.
    // This means we don't need the validator IDs or to do any lookup,
    // we can just zip the iterators.
    let vals_iter = vals_vec.iter();
    let commit_iter = commit_vec.iter();
    for (val, vote_opt) in vals_iter.zip(commit_iter) {
        // skip absent and nil votes
        // NOTE: do we want to check the validity of votes
        // for nil ?
        let vote = match vote_opt {
            Some(v) => v,
            None => continue,
        };

        // check vote is valid from validator
        let sign_bytes = vote.sign_bytes();
        if !val.verify_signature(&sign_bytes, vote.signature()) {
            return Err(Error::InvalidSignature);
        }
        signed_power += val.power();
    }

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
    V: ValidatorSetLookup,
    C: Commit,
    L: TrustLevel,
{
    let total_power = validators.total_power();
    let mut signed_power: u64 = 0;

    // NOTE we don't know the validators that committed this block,
    // so we have to check for each vote if its validator is already known.
    let commit_vec = commit.into_vec();
    let commit_iter = commit_vec.iter();
    for vote_opt in commit_iter {
        // skip absent and nil votes
        // NOTE: do we want to check the validity of votes
        // for nil ?
        let vote = match vote_opt {
            Some(v) => v,
            None => continue,
        };

        // check if this vote is from a known validator
        let val_id = vote.validator_id();
        let val = match validators.validator(val_id) {
            Some(v) => v,
            None => continue,
        };

        // check vote is valid from validator
        let sign_bytes = vote.sign_bytes();

        if !val.verify_signature(&sign_bytes, vote.signature()) {
            return Err(Error::InvalidSignature);
        }
        signed_power += val.power();
    }

    // check the signers account for +1/3 of the voting power (or more if the
    // trust_level requires so)
    if !trust_level.is_enough_power(signed_power, total_power) {
        return Err(Error::InsufficientVotingPower);
    }

    Ok(())
}
