//! Main verification functions that can be used to implement a light client.
//!
//!
//! # Examples
//!
//! ```
//! // TODO: add a proper example maybe showing how a `can_trust_bisection`
//! // looks using the types and methods in this crate/module.
//!```

use crate::lite::{
    Commit, Error, Header, Requester, SignedHeader, Store, TrustThreshold, ValidatorSet,
};
use std::time::{Duration, SystemTime};

/// Returns an error if the header has expired according to the given
/// trusting_period and current time. If so, the verifier must be reset subjectively.
pub fn expired<H>(
    last_header: &H,
    trusting_period: &Duration,
    now: &SystemTime,
) -> Result<(), Error>
where
    H: Header,
{
    match now.duration_since(last_header.bft_time().into()) {
        Ok(passed) => {
            if passed > *trusting_period {
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
pub fn check_support<SH, V, L>(
    h1: &SH,
    h1_next_vals: &V,
    h2: &SH,
    trust_threshold: &L,
    trusting_period: &Duration,
    now: &SystemTime,
) -> Result<(), Error>
where
    SH: SignedHeader,
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
    trust_level: &L,
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

/// Returns Ok if we can trust the header h2 based on h1 otherwise returns an Error.
#[allow(clippy::too_many_arguments)] // TODO: fix this  ...
fn can_trust<SH, VS, L, S, R>(
    h1: &SH,
    h1_next_vals: &VS, // TODO: group these 2 (h1, h1_nex_vals) into one type!
    h2: &SH,
    trust_threshold: &L,
    trusting_period: &Duration,
    now: &SystemTime,
    req: &R,
    store: &mut S,
) -> Result<(), Error>
where
    SH: SignedHeader,
    VS: ValidatorSet,
    L: TrustThreshold,
    S: Store<SignedHeader = SH>,
    R: Requester<SignedHeader = SH>,
{
    match check_support(h1, h1_next_vals, h2, trust_threshold, trusting_period, now) {
        Ok(_) => {
            store.add(h2)?;
            return Ok(());
        }
        Err(e) => {
            if e != Error::InsufficientVotingPower {
                return Err(e);
            }
        }
    }

    let h1_height: u64 = h1.header().height().value();
    let h2_height: u64 = h2.header().height().value();

    let pivot: u64 = (h1_height + h2_height) / 2;
    let hp = req.signed_header(pivot)?;
    let pivot_next_vals = req.validator_set(pivot + 1)?;

    verify(&hp, &pivot_next_vals)?;

    can_trust(
        h1,
        h1_next_vals,
        &hp,
        trust_threshold,
        trusting_period,
        now,
        req,
        store,
    )?;
    store.add(&hp)?;
    can_trust(
        &hp,
        &pivot_next_vals,
        h2,
        trust_threshold,
        trusting_period,
        now,
        req,
        store,
    )?;
    store.add(h2)?;

    Ok(())
}
