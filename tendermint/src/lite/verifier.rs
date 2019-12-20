//! Main verification functions that can be used to implement a light client.
//!
//!
//! # Examples
//!
//! ```
//! // TODO: add a proper example maybe showing how a `can_trust_bisection`
//! // looks using the types and methods in this crate/module.
//!```

use crate::block::Height;
use crate::lite::{
    Commit, Error, Header, Requester, SignedHeader, Store, TrustThreshold, TrustedState,
    ValidatorSet,
};
use std::time::{Duration, SystemTime};

/// Returns an error if the header has expired according to the given
/// trusting_period and current time. If so, the verifier must be reset subjectively.
pub fn is_within_trust_period<H>(
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
/// h2 based on a known trusted state.
/// Note that h1 and h2 have already passed basic validation by calling  `verify`.
/// `Error::InsufficientVotingPower`is returned when there is not enough intersection
/// between validator sets to have skipping condition true.
pub fn check_support<TS, SH, VS, L>(
    trusted_state: &TS,
    h2: &SH,
    trust_threshold: &L,
    trusting_period: &Duration,
    now: &SystemTime,
) -> Result<(), Error>
where
    TS: TrustedState<LastHeader = SH, ValidatorSet = VS>,
    SH: SignedHeader,
    VS: ValidatorSet,
    L: TrustThreshold,
{
    let h1 = trusted_state.last_header();
    let h1_next_vals = trusted_state.validators();

    if let Err(err) = is_within_trust_period(h1.header(), trusting_period, now) {
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

/// Returns Ok if we can trust the passed in (untrusted) header h2
/// based on the given trusted state otherwise returns an Error.
fn can_trust<TS, SH, VS, L, S, R>(
    trusted_state: &TS,    // h1 in spec
    untrusted_header: &SH, // h2 in spec
    trust_threshold: &L,
    trusting_period: &Duration,
    now: &SystemTime,
    req: &R,
    store: &mut S,
) -> Result<(), Error>
where
    TS: TrustedState<LastHeader = SH, ValidatorSet = VS>,
    SH: SignedHeader,
    VS: ValidatorSet,
    L: TrustThreshold,
    S: Store<SignedHeader = SH>,
    R: Requester<SignedHeader = SH, ValidatorSet = VS>,
{
    // can we trust the still untrusted header based on the given trusted state?
    match check_support(
        trusted_state,
        untrusted_header,
        trust_threshold,
        trusting_period,
        now,
    ) {
        Ok(_) => {
            store.add(untrusted_header)?;
            return Ok(());
        }
        Err(e) => {
            if e != Error::InsufficientVotingPower {
                return Err(e);
            }
        }
    }

    // Here we can't trust the passed in untrusted header based on the known trusted state.
    // Run bisection: try again with a pivot header whose height is in the
    // middle of the trusted height and the desired height (h2.height).

    let trusted_header = trusted_state.last_header();
    let trusted_height: u64 = trusted_header.header().height().value();
    let untrusted_height: u64 = untrusted_header.header().height().value();
    let pivot: u64 = (trusted_height + untrusted_height) / 2;
    let pivot_header = req.signed_header(pivot)?;
    let pivot_next_vals = req.validator_set(pivot + 1)?;

    verify(&pivot_header, &pivot_next_vals)?;

    // Can we trust pivot header based on trusted_state?
    can_trust(
        trusted_state,
        &pivot_header,
        trust_threshold,
        trusting_period,
        now,
        req,
        store,
    )?;
    // Trust the header in between the trusted and (still) untrusted height:
    store.add(&pivot_header)?;
    let pivot_trusted = TS::new(pivot_header, pivot_next_vals);

    // Can we trust h2 based on the (now trusted) "pivot header"?
    can_trust(
        &pivot_trusted,
        untrusted_header,
        trust_threshold,
        trusting_period,
        now,
        req,
        store,
    )?;
    // Add to trusted headers:
    store.add(untrusted_header)?;

    Ok(())
}

/// This function captures the high level logic of the light client verification, i.e.,
/// an application call to the light client module to (optionally download) and
/// verify a header for some height.
pub fn verify_header<TS, SH, VS, L, S, R>(
    height: Height,
    trust_threshold: &L,
    trusting_period: &Duration,
    now: &SystemTime,
    req: &R,
    store: &mut S,
) -> Result<(), Error>
where
    TS: TrustedState<LastHeader = SH, ValidatorSet = VS>,
    L: TrustThreshold,
    S: Store<SignedHeader = SH>,
    R: Requester<SignedHeader = SH, ValidatorSet = VS>,
    VS: ValidatorSet,
    SH: SignedHeader,
{
    // Check if we already trusted a header at the given height and it didn't expire:
    if let Ok(sh2) = store.get(height) {
        is_within_trust_period(sh2.header(), trusting_period, now)?
    }

    // We haven't trusted a header at height yet. Request it:
    let sh2 = req.signed_header(height)?;
    let sh2_next_vals = req.validator_set(height.increment())?;
    verify(&sh2, &sh2_next_vals)?;
    is_within_trust_period(sh2.header(), trusting_period, now)?;

    // Get the highest trusted header with height lower than sh2's.
    let sh1 = store.get_smaller_or_equal(height)?;
    // TODO(Liamsi): it's odd that we need re-request the validator set here; shouldn't this
    // be stored instead?!
    let sh1_next_vals = req.validator_set(sh1.header().height().value() + 1)?;
    let sh1_trusted = TS::new(sh1, sh1_next_vals);

    can_trust(
        &sh1_trusted,
        &sh2,
        trust_threshold,
        trusting_period,
        now,
        req,
        store,
    )?;

    is_within_trust_period(sh2.header(), trusting_period, now)?;
    store.add(&sh2)?;

    Ok(())
}
