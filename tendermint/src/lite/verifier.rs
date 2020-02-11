//! Main verification functions that can be used to implement a light client.
//!
//!
//! # Examples
//!
//! ```
//! // TODO: add a proper example maybe showing how a `can_trust_bisection`
//! // looks using the types and methods in this crate/module.
//! ```

use std::cmp::Ordering;
use std::time::{Duration, SystemTime};

use crate::lite::{
    Commit, Error, Header, Height, Requester, SignedHeader, TrustThreshold, TrustedState,
    ValidatorSet,
};

/// Returns an error if the header has expired according to the given
/// trusting_period and current time. If so, the verifier must be reset subjectively.
fn is_within_trust_period<H>(
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

/// Validate the validators, next validators, against the signed header.
/// This is equivalent to validateSignedHeaderAndVals in the spec.
fn validate<C, H>(
    signed_header: &SignedHeader<C, H>,
    vals: &C::ValidatorSet,
    next_vals: &C::ValidatorSet,
) -> Result<(), Error>
where
    C: Commit,
    H: Header,
{
    let header = signed_header.header();
    let commit = signed_header.commit();

    // ensure the header validator hashes match the given validators
    if header.validators_hash() != vals.hash() {
        return Err(Error::InvalidValidatorSet);
    }
    if header.next_validators_hash() != next_vals.hash() {
        return Err(Error::InvalidNextValidatorSet);
    }

    // ensure the header matches the commit
    if header.hash() != commit.header_hash() {
        return Err(Error::InvalidCommitValue);
    }

    // additional implementation specific validation:
    commit.validate(vals)?;

    Ok(())
}

/// Verify that +2/3 of the correct validator set signed this commit.
/// NOTE: These validators are expected to be the correct validators for the commit,
/// but since we're using voting_power_in, we can't actually detect if there's
/// votes from validators not in the set.
fn verify_commit_full<C>(vals: &C::ValidatorSet, commit: &C) -> Result<(), Error>
where
    C: Commit,
{
    let total_power = vals.total_power();
    let signed_power = commit.voting_power_in(vals)?;

    // check the signers account for +2/3 of the voting power
    if signed_power * 3 <= total_power * 2 {
        return Err(Error::InvalidCommit);
    }

    Ok(())
}

/// Verify that +1/3 of the given validator set signed this commit.
/// NOTE the given validators do not necessarily correspond to the validator set for this commit,
/// but there may be some intersection. The trust_level parameter allows clients to require more
/// than +1/3 by implementing the TrustLevel trait accordingly.
fn verify_commit_trusting<C, L>(
    validators: &C::ValidatorSet,
    commit: &C,
    trust_level: L,
) -> Result<(), Error>
where
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

// Verify a single untrusted header against a trusted state.
// Includes all validation and signature verification.
// Not publicly exposed since it does not check for expiry
// and hence it's possible to use it incorrectly.
// If trusted_state is not expired and this returns Ok, the
// untrusted_sh and untrusted_next_vals can be considered trusted.
fn verify_single_inner<H, C, L>(
    trusted_state: &TrustedState<C, H>,
    untrusted_sh: &SignedHeader<C, H>,
    untrusted_vals: &C::ValidatorSet,
    untrusted_next_vals: &C::ValidatorSet,
    trust_threshold: L,
) -> Result<(), Error>
where
    H: Header,
    C: Commit,
    L: TrustThreshold,
{
    // validate the untrusted header against its commit, vals, and next_vals
    let untrusted_header = untrusted_sh.header();
    let untrusted_commit = untrusted_sh.commit();

    validate(untrusted_sh, untrusted_vals, untrusted_next_vals)?;

    // ensure the new height is higher.
    // if its +1, ensure the vals are correct.
    // if its >+1, ensure we can skip to it
    let trusted_header = trusted_state.last_header().header();
    let trusted_height = trusted_header.height();
    let untrusted_height = untrusted_sh.header().height();

    // ensure the untrusted_header.bft_time() > trusted_header.bft_time()
    if untrusted_header.bft_time().into() <= trusted_header.bft_time().into() {
        return Err(Error::NonIncreasingTime);
    }

    match untrusted_height.cmp(&trusted_height.checked_add(1).expect("height overflow")) {
        Ordering::Less => return Err(Error::NonIncreasingHeight),
        Ordering::Equal => {
            let trusted_vals_hash = trusted_header.next_validators_hash();
            let untrusted_vals_hash = untrusted_header.validators_hash();
            if trusted_vals_hash != untrusted_vals_hash {
                // TODO: more specific error
                // ie. differentiate from when next_vals.hash() doesnt
                // match the header hash ...
                return Err(Error::InvalidNextValidatorSet);
            }
        }
        Ordering::Greater => {
            let trusted_vals = trusted_state.validators();
            verify_commit_trusting(trusted_vals, untrusted_commit, trust_threshold)?;
        }
    }

    // All validation passed successfully. Verify the validators correctly committed the block.
    verify_commit_full(untrusted_vals, untrusted_sh.commit())
}

/// Verify a single untrusted header against a trusted state.
/// Ensures our last trusted header hasn't expired yet, and that
/// the untrusted header can be verified using only our latest trusted
/// state from the store.
///
/// On success, the caller is responsible for updating the store with the returned
/// header to be trusted.
///
/// This function is primarily for use by IBC handlers.
pub fn verify_single<H, C, L>(
    trusted_state: TrustedState<C, H>,
    untrusted_sh: &SignedHeader<C, H>,
    untrusted_vals: &C::ValidatorSet,
    untrusted_next_vals: &C::ValidatorSet,
    trust_threshold: L,
    trusting_period: &Duration,
    now: &SystemTime,
) -> Result<TrustedState<C, H>, Error>
where
    H: Header,
    C: Commit,
    L: TrustThreshold,
{
    // Fetch the latest state and ensure it hasn't expired.
    let trusted_sh = trusted_state.last_header();
    is_within_trust_period(trusted_sh.header(), trusting_period, now)?;

    verify_single_inner(
        &trusted_state,
        untrusted_sh,
        untrusted_vals,
        untrusted_next_vals,
        trust_threshold,
    )?;

    // The untrusted header is now trusted;
    // return to the caller so they can update the store:
    Ok(TrustedState::new(untrusted_sh, untrusted_next_vals))
}

/// Attempt to "bisect" from the passed-in trusted state (with header of height h)
/// to the given untrusted height (h+n) by requesting the necessary
/// data (signed headers and validators from height (h, h+n]).
///
/// On success, callers are responsible for storing the returned states
/// which can now be trusted.
///
/// Returns an error if:
///     - we're already at or past that height
///     - our latest state expired
///     - any requests fail
///     - requested data is inconsistent (eg. vals don't match hashes in header)
///     - validators did not correctly commit their blocks
///
/// This function is recursive: it uses a bisection algorithm
/// to request data for intermediate heights as necessary.
/// Ensures our last trusted header hasn't expired yet, and that
/// data from the untrusted height can be verified, possibly using
/// data from intermediate heights.
///
/// This function is primarily for use by a light node.
pub fn verify_bisection<C, H, L, R>(
    trusted_state: TrustedState<C, H>,
    untrusted_height: Height,
    trust_threshold: L,
    trusting_period: &Duration,
    now: &SystemTime,
    req: &R,
) -> Result<Vec<TrustedState<C, H>>, Error>
where
    H: Header,
    C: Commit,
    L: TrustThreshold,
    R: Requester<C, H>,
{
    // Ensure the latest state hasn't expired.
    // Note we only check for expiry once in this
    // verify_and_update_bisection function since we assume the
    // time is passed in and we don't access a clock internally.
    // Thus the trust_period must be long enough to incorporate the
    // expected time to complete this function.
    let trusted_sh = trusted_state.last_header();
    is_within_trust_period(trusted_sh.header(), trusting_period, now)?;

    // TODO: consider fetching the header we're trying to sync to and
    // checking that it's time is less then `now + X` for some small X.
    // If not, it means that either our local clock is really slow
    // or the blockchains BFT time is really wrong.
    // In either case, we should probably raise an error.
    // Note this would be stronger than checking that the untrusted
    // header is within the trusting period, as it could still diverge
    // significantly from `now`.
    // NOTE: we actually have to do this for every header we fetch,
    // We do check bft_time is monotonic, but that check might happen too late.
    // So every header we fetch must be checked to be less than now+X

    // this is only used to memoize intermediate trusted states:
    let mut cache: Vec<TrustedState<C, H>> = Vec::new();
    // inner recursive function which assumes
    // trusting_period check is already done.
    verify_bisection_inner(
        &trusted_state,
        untrusted_height,
        trust_threshold,
        req,
        &mut cache,
    )?;
    // return all intermediate trusted states up to untrusted_height
    Ok(cache)
}

// inner recursive function for verify_and_update_bisection.
// see that function's docs.
// A cache is passed in to memoize all new states to be trusted.
// Note: we only write to the cache and it guarantees that we do
// not store states twice.
// Additionally, a new state is returned for convenience s.t. it can
// be used for the other half of the recursion.
fn verify_bisection_inner<H, C, L, R>(
    trusted_state: &TrustedState<C, H>,
    untrusted_height: Height,
    trust_threshold: L,
    req: &R,
    mut cache: &mut Vec<TrustedState<C, H>>,
) -> Result<TrustedState<C, H>, Error>
where
    H: Header,
    C: Commit,
    L: TrustThreshold,
    R: Requester<C, H>,
{
    // fetch the header and vals for the new height
    let untrusted_sh = &req.signed_header(untrusted_height)?;
    let untrusted_vals = &req.validator_set(untrusted_height)?;
    let untrusted_next_vals =
        &req.validator_set(untrusted_height.checked_add(1).expect("height overflow"))?;

    // check if we can skip to this height and if it verifies.
    match verify_single_inner(
        trusted_state,
        untrusted_sh,
        untrusted_vals,
        untrusted_next_vals,
        trust_threshold,
    ) {
        Ok(_) => {
            // Successfully verified!
            // memoize the new to be trusted state and return.
            let ts = TrustedState::new(untrusted_sh, untrusted_next_vals);
            cache.push(ts.clone());
            return Ok(ts);
        }
        Err(e) => {
            // If something went wrong, return the error.
            if e != Error::InsufficientVotingPower {
                return Err(e);
            }

            // Insufficient voting power to update.
            // Engage bisection, below.
        }
    }

    // Get the pivot height for bisection.
    let trusted_h = trusted_state.last_header().header().height();
    let untrusted_h = untrusted_height;
    let pivot_height = trusted_h.checked_add(untrusted_h).expect("height overflow") / 2;

    // Recursive call to bisect to the pivot height.
    // When this completes, we will either return an error or
    // have updated the cache to the pivot height.
    let trusted_left = verify_bisection_inner(
        trusted_state,
        pivot_height,
        trust_threshold,
        req,
        &mut cache,
    )?;

    // Recursive call to update to the original untrusted_height.
    verify_bisection_inner(
        &trusted_left,
        untrusted_height,
        trust_threshold,
        req,
        &mut cache,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lite::mocks::*;
    use crate::lite::TrustThresholdFraction;

    type MockState = TrustedState<MockCommit, MockHeader>;

    // start all blockchains from here ...
    fn init_time() -> SystemTime {
        SystemTime::UNIX_EPOCH
    }

    // create an initial trusted state from the given vals
    fn init_trusted_state(
        vals_vec: Vec<usize>,
        commit_vec: Vec<usize>,
        next_vals_vec: Vec<usize>,
        height: u64,
    ) -> MockState {
        // time has to be increasing:
        let time = init_time() + Duration::new(height * 2, 0);
        let vals = &MockValSet::new(vals_vec.clone());
        let next_vals = &MockValSet::new(next_vals_vec);
        let header = MockHeader::new(height, time, vals.hash(), next_vals.hash());
        let commit = MockCommit::new(header.hash(), commit_vec);
        let sh = &MockSignedHeader::new(commit, header);
        MockState::new(sh, vals)
    }

    // create the next state with the given vals and commit.
    fn next_state(
        vals_vec: Vec<usize>,
        commit_vec: Vec<usize>,
    ) -> (MockSignedHeader, MockValSet, MockValSet) {
        let time = init_time() + Duration::new(10, 0);
        let height = 10;
        let vals = MockValSet::new(vals_vec);
        let next_vals = vals.clone();
        let header = MockHeader::new(height, time, vals.hash(), next_vals.hash());
        let commit = MockCommit::new(header.hash(), commit_vec);
        (MockSignedHeader::new(commit, header), vals, next_vals)
    }


    // TODO: find a better name
    #[derive(Clone)]
    struct ValsAndCommit {
        vals_vec: Vec<usize>,
        commit_vec: Vec<usize>,
    }

    impl ValsAndCommit {
        pub fn new(vals_vec: Vec<usize>, commit_vec: Vec<usize>) -> ValsAndCommit {
            ValsAndCommit {vals_vec, commit_vec}
        }
    }
    // Init a mock Requester.
    // For each pair of lists of validators (cur and next vals) we
    // init a trusted state (signed header and vals);
    // Note: for bisection up-to height n, provide n+2 validators as validators for n+1
    // will be requested to verify height n.
    fn init_requester(vals_and_commit_for_height: Vec<ValsAndCommit>) -> MockRequester {
        let mut req = MockRequester::new();
        let max_height = vals_and_commit_for_height.len() as u64;
        for (h, vac) in vals_and_commit_for_height.iter().enumerate() {
            let height = (h + 1) as u64; // height starts with 1 ...
            if height < max_height {
                let next_vals = vals_and_commit_for_height.get(h + 1).expect("next_vals missing").vals_vec.clone();
                let ts = &init_trusted_state(vac.vals_vec.to_owned(), vac.commit_vec.to_owned(), next_vals.to_owned(), height);
                req.signed_headers.insert(height, ts.last_header().clone());
                req.validators.insert(height, ts.validators().to_owned());
            }
        }
        req
    }

    // make a state with the given vals and commit and ensure we get the error.
    fn assert_single_err(
        ts: &TrustedState<MockCommit, MockHeader>,
        vals: Vec<usize>,
        commit: Vec<usize>,
        err: Error,
    ) {
        let (un_sh, un_vals, un_next_vals) = next_state(vals, commit);
        let result = verify_single_inner(
            ts,
            &un_sh,
            &un_vals,
            &un_next_vals,
            TrustThresholdFraction::default(),
        );
        assert_eq!(result, Err(err));
    }

    // make a state with the given vals and commit and ensure we get no error.
    fn assert_single_ok(ts: &MockState, vals: Vec<usize>, commit: Vec<usize>) {
        let (un_sh, un_vals, un_next_vals) = next_state(vals, commit);
        assert!(verify_single_inner(
            ts,
            &un_sh,
            &un_vals,
            &un_next_vals,
            TrustThresholdFraction::default()
        )
        .is_ok());
    }

    // use the sequence of states with the given vals for the requester
    // and ensure bisection yields no error.
    fn assert_bisection_ok(
        req: &MockRequester,
        ts: &TrustedState<MockCommit, MockHeader>,
        untrusted_height: u64,
        expected_num_of_requests: usize,
        expected_final_state: &MockState,
    ) {
        let mut cache: Vec<MockTrustedState> = Vec::new();
        let ts_new = verify_bisection_inner(
            &ts,
            untrusted_height,
            TrustThresholdFraction::default(),
            req,
            cache.as_mut(),
        )
        .expect("should have passed");
        assert_eq!(ts_new, expected_final_state.to_owned());
        assert_eq!(cache.len(), expected_num_of_requests);
        assert_uniqueness(cache);
    }

    fn assert_uniqueness(cache: Vec<TrustedState<MockCommit, MockHeader>>) {
        let mut uniq = cache.clone();
        uniq.dedup();
        assert_eq!(cache, uniq);
    }

    // use the sequence of states with the given vals for the requester
    // and ensure we get the expected error.
    fn assert_bisection_err(
        req: &MockRequester,
        ts: &TrustedState<MockCommit, MockHeader>,
        untrusted_height: u64,
        err: Error,
    ) {
        let mut cache: Vec<MockTrustedState> = Vec::new();
        let result = verify_bisection_inner(
            &ts,
            untrusted_height,
            TrustThresholdFraction::default(),
            req,
            cache.as_mut(),
        );
        assert_eq!(result, Err(err));
    }

    // valid to skip, but invalid commit. 1 validator.
    #[test]
    fn test_verify_single_skip_1_val_verify() {
        let ts = &init_trusted_state(vec![0],vec![0], vec![0], 1);

        // 100% overlap, but wrong commit.
        // NOTE: This should be an invalid commit error since there's
        // a vote from a validator not in the set!
        // but voting_power_in isn't smart enough to see this ...
        // TODO(ismail): https://github.com/interchainio/tendermint-rs/issues/140
        assert_single_err(ts, vec![1], vec![0], Error::InvalidCommit);
    }

    // valid commit and data, starting with 1 validator.
    // test if we can skip to it.
    #[test]
    fn test_verify_single_skip_1_val_skip() {
        let ts = &init_trusted_state(vec![0], vec![0], vec![0], 1);
        let err = Error::InsufficientVotingPower;

        //*****
        // Ok

        // 100% overlap (original signer is present in commit)
        assert_single_ok(ts, vec![0], vec![0]);
        assert_single_ok(ts, vec![0, 1], vec![0, 1]);
        assert_single_ok(ts, vec![0, 1, 2], vec![0, 1, 2]);
        assert_single_ok(ts, vec![0, 1, 2, 3], vec![0, 1, 2, 3]);

        //*****
        // Err

        // 0% overlap - new val set without the original signer
        assert_single_err(ts, vec![1], vec![1], err);

        // 0% overlap - val set contains original signer, but they didn't sign
        assert_single_err(ts, vec![0, 1, 2, 3], vec![1, 2, 3], err);
    }

    // valid commit and data, starting with 2 validators.
    // test if we can skip to it.
    #[test]
    fn test_verify_single_skip_2_val_skip() {
        let ts = &init_trusted_state(vec![0, 1], vec![0, 1],vec![0, 1], 1);
        let err = Error::InsufficientVotingPower;

        //*************
        // OK

        // 100% overlap (both original signers still present)
        assert_single_ok(ts, vec![0, 1], vec![0, 1]);
        assert_single_ok(ts, vec![0, 1, 2], vec![0, 1, 2]);

        // 50% overlap (one original signer still present)
        assert_single_ok(ts, vec![0], vec![0]);
        assert_single_ok(ts, vec![0, 1, 2, 3], vec![1, 2, 3]);

        //*************
        // Err

        // 0% overlap (neither original signer still present)
        assert_single_err(ts, vec![2], vec![2], err);

        // 0% overlap (original signer is still in val set but not in commit)
        assert_single_err(ts, vec![0, 2, 3, 4], vec![2, 3, 4], err);
    }

    // valid commit and data, starting with 3 validators.
    // test if we can skip to it.
    #[test]
    fn test_verify_single_skip_3_val_skip() {
        let ts = &init_trusted_state(vec![0, 1, 2], vec![0, 1, 2], vec![0, 1, 2], 1);
        let err = Error::InsufficientVotingPower;

        //*************
        // OK

        // 100% overlap (both original signers still present)
        assert_single_ok(ts, vec![0, 1, 2], vec![0, 1, 2]);
        assert_single_ok(ts, vec![0, 1, 2, 3], vec![0, 1, 2, 3]);

        // 66% overlap (two original signers still present)
        assert_single_ok(ts, vec![0, 1], vec![0, 1]);
        assert_single_ok(ts, vec![0, 1, 2, 3], vec![1, 2, 3]);

        //*************
        // Err

        // 33% overlap (one original signer still present)
        assert_single_err(ts, vec![0], vec![0], err);
        assert_single_err(ts, vec![0, 3], vec![0, 3], err);

        // 0% overlap (neither original signer still present)
        assert_single_err(ts, vec![3], vec![2], err);

        // 0% overlap (original signer is still in val set but not in commit)
        assert_single_err(ts, vec![0, 3, 4, 5], vec![3, 4, 5], err);
    }

    #[test]
    fn test_verify_bisection_1_val() {
        let final_state = init_trusted_state(vec![0], vec![0], vec![0], 2);
        let vac = ValsAndCommit::new(vec![0], vec![0]);
        let req = init_requester(vec![vac.clone(), vac.clone(), vac.clone(), vac.clone()]);
        let sh = req.signed_header(1).expect("first sh not present");
        let vals = req.validator_set(1).expect("init. valset not present");
        let ts = &MockState::new(&sh, &vals);

        assert_bisection_ok(&req, &ts, 2, 1, &final_state);

        let final_state = init_trusted_state(vec![0], vec![0], vec![0], 3);
        let vac = ValsAndCommit::new(vec![0], vec![0]);
        let req = init_requester(vec![vac.clone(),vac.clone(),vac.clone(),vac.clone(),vac.clone()]);
        assert_bisection_ok(&req, &ts, 3, 1, &final_state);
    }


    #[test]
    fn test_verify_bisection() {
        //*************
        // OK

        let mut vals_and_commit_for_height: Vec<ValsAndCommit> = Vec::new();
        vals_and_commit_for_height.push(ValsAndCommit::new(vec![0,1,2,3,4,5], vec![0,1,2,3,4,5])); // 1
        vals_and_commit_for_height.push(ValsAndCommit::new(vec![0,1,2], vec![0,1,2])); // 2 -> 50% val change
        vals_and_commit_for_height.push(ValsAndCommit::new(vec![0,1], vec![0,1])); // 3 -> 33% change
        vals_and_commit_for_height.push(ValsAndCommit::new(vec![1,2], vec![1,2])); // 4 -> 50% change
        vals_and_commit_for_height.push(ValsAndCommit::new(vec![0,2], vec![0,2])); // 5 -> 50% <- too much change (from 1), need to bisect...
        vals_and_commit_for_height.push(ValsAndCommit::new(vec![0,2], vec![0,2])); // 6 -> (only needed to validate 5)
        vals_and_commit_for_height.push(ValsAndCommit::new(vec![0,2], vec![0,2])); // 7 -> (only used to construct state 6)
        let final_ts = init_trusted_state(vec![0, 2], vec![0, 2], vec![0, 2], 5);
        let req = init_requester(vals_and_commit_for_height);
        let sh = req.signed_header(1).expect("first sh not present");
        let vals = req.validator_set(1).expect("init. valset not present");
        let ts = &MockState::new(&sh, &vals);

        assert_bisection_ok(&req, &ts, 5, 3, &final_ts);

        //*************
        // Err

        // fails due to missing vals for height 6:
        let mut faulty_req = req;
        faulty_req.validators.remove(&6_u64);
        assert_bisection_err(&faulty_req, &ts, 5, Error::RequestFailed);

        // Error: can't bisect from trusted height 1 to height 1
        // (here because non-increasing time is caught first)
        let vac = ValsAndCommit::new(vec![0,1,2], vec![0,1,2]);
        let req = init_requester(vec![vac.clone(), vac.clone(), vac.clone()]);
        assert_bisection_err(&req, &ts, 1, Error::NonIncreasingTime);

        // can't bisect from trusted height 1 to height 1 (here we tamper with time but
        // expect to fail on NonIncreasingHeight):
        let vac = ValsAndCommit::new(vec![0,1,2], vec![0,1,2]);
        let mut req = init_requester(vec![vac.clone(),vac.clone(),vac.clone()]);
        let sh = req.signed_headers.get(&1_u64).unwrap();
        let mut time_tampered_header = sh.header().clone();
        time_tampered_header.set_time(init_time() + Duration::new(5, 0));
        let tampered_commit = MockCommit::new(time_tampered_header.hash(), vec![0, 1, 2]);
        let new_sh = MockSignedHeader::new(tampered_commit, time_tampered_header);
        // replace the signed header:
        req.signed_headers.insert(1, new_sh);
        assert_bisection_err(&req, &ts, 1, Error::NonIncreasingHeight);
    }

    #[test]
    fn test_is_within_trust_period() {
        let header_time = SystemTime::UNIX_EPOCH;
        let period = Duration::new(100, 0);
        let now = header_time + Duration::new(10, 0);

        // less than the period, OK
        let header = MockHeader::new(4, header_time, fixed_hash(), fixed_hash());
        assert!(is_within_trust_period(&header, &period, &now).is_ok());

        // equal to the period, OK
        let now = header_time + period;
        assert!(is_within_trust_period(&header, &period, &now).is_ok());

        // greater than the period, not OK
        let now = header_time + period + Duration::new(1, 0);
        assert!(is_within_trust_period(&header, &period, &now).is_err());

        // bft time in header is later than now, not OK:
        let now = SystemTime::UNIX_EPOCH;
        let later_than_now = now + Duration::new(60, 0);
        let future_header = MockHeader::new(4, later_than_now, fixed_hash(), fixed_hash());
        assert!(is_within_trust_period(&future_header, &period, &now).is_err());
    }
}
