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

use crate::block::Height;
use crate::lite::{
    Commit, Error, Header, Requester, SignedHeader, Store, TrustThreshold, TrustedState,
    ValidatorSet,
};

/// Returns an error if the header has expired according to the given
/// `trusting_period` and current time. If so, the verifier must be reset subjectively.
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

/// Validate the validators, next validators, and commit against the header.
// TODO(EB): consider making this a method on Commit so the details are hidden,
// and so we can remove the votes_len() method (that check would be part of the
// methods implementation). These checks aren't reflected
// explicitly in the spec yet, only in the sentence "Additional checks should
// be done in the implementation to ensure header is well formed".
fn validate_untrusted<H, V, C>(header: &H, commit: &C, vals: &V, next_vals: &V) -> Result<(), Error>
where
    H: Header,
    V: ValidatorSet,
    C: Commit,
{
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

    // ensure the validator size matches the commit size
    // NOTE: this is commit structure specific and should be
    // hidden from the light client ...
    if vals.len() != commit.votes_len() {
        return Err(Error::InvalidCommitLength);
    }

    Ok(())
}

/// Verify that +2/3 of the correct validator set signed this commit.
/// NOTE: These validators are expected to be the correct validators for the commit,
/// but since we're using `voting_power_in`, we can't actually detect if there's
/// votes from validators not in the set.
pub fn verify_commit_full<C>(vals: &C::ValidatorSet, commit: &C) -> Result<(), Error>
where
    C: Commit,
{
    let total_power = vals.total_power();
    let signed_power = commit.voting_power_in(vals)?;

    // check the signers account for +2/3 of the voting power
    if signed_power * 3 <= total_power * 2 {
        // TODO(EB): Use a different error from
        // verify_commit_trusting else bisection
        // will happen when the commit is actually just invalid!
        return Err(Error::InsufficientVotingPower);
    }

    Ok(())
}

/// Verify that +1/3 of the given validator set signed this commit.
/// NOTE the given validators do not necessarily correspond to the validator set for this commit,
/// but there may be some intersection. The `trust_level` parameter allows clients to require more
/// than +1/3 by implementing the `TrustLevel` trait accordingly.
pub fn verify_commit_trusting<C, L>(
    validators: &C::ValidatorSet,
    commit: &C,
    trust_level: &L,
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
pub fn verify_single<TS, SH, C, L>(
    trusted_state: &TS,
    untrusted_sh: &SH,
    untrusted_vals: &C::ValidatorSet,
    untrusted_next_vals: &C::ValidatorSet,
    trust_threshold: &L,
) -> Result<(), Error>
where
    TS: TrustedState<LastHeader = SH, ValidatorSet = C::ValidatorSet>,
    SH: SignedHeader<Commit = C>,
    C: Commit,
    L: TrustThreshold,
{
    // validate the untrusted header against its commit, vals, and next_vals
    let untrusted_header = untrusted_sh.header();
    let untrusted_commit = untrusted_sh.commit();

    validate_untrusted(
        untrusted_header,
        untrusted_commit,
        untrusted_vals,
        untrusted_next_vals,
    )?;

    // ensure the new height is higher.
    // if its +1, ensure the vals are correct.
    // if its >+1, ensure we can skip to it
    let trusted_header = trusted_state.last_header().header();
    let trusted_height = trusted_header.height();
    let untrusted_height = untrusted_sh.header().height();

    // TODO: ensure the untrusted_header.bft_time() > trusted_header.bft_time()

    match untrusted_height.cmp(&trusted_height.increment()) {
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

/// Attempt to update the store to the given untrusted header.
/// Ensures our last trusted header hasn't expired yet, and that
/// the untrusted header can be verified using only our latest trusted
/// state from the store.
/// This function is primarily for use by IBC handlers.
pub fn verify_and_update_single<TS, SH, C, L, S>(
    untrusted_sh: &SH,
    untrusted_vals: &C::ValidatorSet,
    untrusted_next_vals: &C::ValidatorSet,
    trust_threshold: &L,
    trusting_period: &Duration,
    now: &SystemTime,
    store: &mut S,
) -> Result<(), Error>
where
    TS: TrustedState<LastHeader = SH, ValidatorSet = C::ValidatorSet>,
    SH: SignedHeader<Commit = C>,
    C: Commit,
    L: TrustThreshold,
    S: Store<TrustedState = TS>,
{
    // Fetch the latest state and ensure it hasn't expired.
    let trusted_state = store.get(Height::from(0))?;
    let trusted_sh = trusted_state.last_header();
    is_within_trust_period(trusted_sh.header(), trusting_period, now)?;

    verify_single(
        trusted_state,
        untrusted_sh,
        untrusted_vals,
        untrusted_next_vals,
        trust_threshold,
    )?;

    // The untrusted header is now trusted; Update the store
    let new_trusted_state = TS::new(untrusted_sh, untrusted_next_vals);
    store.add(&new_trusted_state)
}

/// Attempt to update the store to the given untrusted height
/// by requesting the necessary data (signed headers and validators).
/// Returns an error if:
///     - we're already at or past that height
///     - our latest state expired
///     - any requests fail
///     - requested data is inconsistent (eg. vals don't match hashes in header)
///     - validators did not correctly commit their blocks
/// This function is recursive: it uses a bisection algorithm
/// to request data for intermediate heights as necessary.
/// Ensures our last trusted header hasn't expired yet, and that
/// data from the untrusted height can be verified, possibly using
/// data from intermediate heights.
/// This function is primarily for use by a light node.
pub fn verify_and_update_bisection<TS, SH, C, L, R, S>(
    untrusted_height: Height,
    trust_threshold: &L,
    trusting_period: &Duration,
    now: &SystemTime,
    req: &R,
    store: &mut S,
) -> Result<(), Error>
where
    TS: TrustedState<LastHeader = SH, ValidatorSet = C::ValidatorSet>,
    SH: SignedHeader<Commit = C>,
    C: Commit,
    L: TrustThreshold,
    R: Requester<SignedHeader = SH, ValidatorSet = C::ValidatorSet>,
    S: Store<TrustedState = TS>,
{
    // Fetch the latest state and ensure it hasn't expired.
    // Note we only check for expiry once in this
    // verify_and_update_bisection function since we assume the
    // time is passed in and we don't access a clock internall.
    // Thus the trust_period must be long enough to incorporate the
    // expected time to complete this function.
    let trusted_state = store.get(Height::from(0))?;
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

    // inner recursive function which assumes
    // trusting_period check is already done.
    _verify_and_update_bisection(untrusted_height, trust_threshold, req, store)
}

// inner recursive function for verify_and_update_bisection.
// see that function's docs.
fn _verify_and_update_bisection<TS, SH, C, L, R, S>(
    untrusted_height: Height,
    trust_threshold: &L,
    req: &R,
    store: &mut S,
) -> Result<(), Error>
where
    TS: TrustedState<LastHeader = SH, ValidatorSet = C::ValidatorSet>,
    SH: SignedHeader<Commit = C>,
    C: Commit,
    L: TrustThreshold,
    R: Requester<SignedHeader = SH, ValidatorSet = C::ValidatorSet>,
    S: Store<TrustedState = TS>,
{
    // this get is redundant the first time.
    // TODO: possibly refactor so this func takes and returns
    // trusted_state.
    let trusted_state = store.get(Height::from(0))?;
    let trusted_sh = trusted_state.last_header();

    // fetch the header and vals for the new height
    let untrusted_sh = &req.signed_header(untrusted_height)?;
    let untrusted_vals = &req.validator_set(untrusted_height)?;
    let untrusted_next_vals = &req.validator_set(untrusted_height.increment())?;

    // check if we can skip to this height and if it verifies.
    match verify_single(
        trusted_state,
        untrusted_sh,
        untrusted_vals,
        untrusted_next_vals,
        trust_threshold,
    ) {
        Ok(_) => {
            // Successfully verified!
            // Trust the new state and return.
            let new_trusted_state = TS::new(untrusted_sh, untrusted_next_vals);
            return store.add(&new_trusted_state);
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
    let trusted_height = trusted_sh.header().height().value();
    let pivot_height = Height::from((trusted_height + untrusted_height.value()).wrapping_div(2));

    // Recursive call to update to the pivot height.
    // When this completes, we will either return an error or
    // have updated the store to the pivot height.
    _verify_and_update_bisection(pivot_height, trust_threshold, req, store)?;

    // Recursive call to update to the original untrusted_height.
    _verify_and_update_bisection(untrusted_height, trust_threshold, req, store)
}

#[allow(clippy::panic)]
#[cfg(test)]
mod tests {
    use serde::Serialize;
    use sha2::{Digest, Sha256};

    use crate::{hash::Algorithm, Hash};

    use super::*;

    #[derive(Clone, Debug, Serialize)]
    struct MockHeader {
        height: u64,
        time: SystemTime,
        vals: Hash,
        next_vals: Hash,
    }

    impl MockHeader {
        const fn new(height: u64, time: SystemTime, vals: Hash, next_vals: Hash) -> Self {
            Self {
                height,
                time,
                vals,
                next_vals,
            }
        }
    }

    impl Header for MockHeader {
        type Time = SystemTime;

        fn height(&self) -> Height {
            Height::from(self.height)
        }
        fn bft_time(&self) -> Self::Time {
            self.time
        }
        fn validators_hash(&self) -> Hash {
            self.vals
        }
        fn next_validators_hash(&self) -> Hash {
            self.next_vals
        }
        fn hash(&self) -> Hash {
            json_hash(self)
        }
    }

    fn json_hash<T: ?Sized + Serialize>(value: &T) -> Hash {
        let encoded = serde_json::to_vec(value).expect("Hash serialization failed");
        let hashed = Sha256::digest(&encoded);
        Hash::new(Algorithm::Sha256, &hashed).expect("unable to create Hash")
    }

    // vals are just ints, each has power 1
    #[derive(Clone, Debug, Serialize)]
    struct MockValSet {
        // NOTE: use HashSet instead?
        vals: Vec<usize>,
    }

    impl MockValSet {
        fn new(vals: Vec<usize>) -> Self {
            Self { vals }
        }
    }

    impl ValidatorSet for MockValSet {
        fn hash(&self) -> Hash {
            json_hash(&self)
        }
        fn total_power(&self) -> u64 {
            self.vals.len() as u64
        }
        fn len(&self) -> usize {
            self.vals.len()
        }
        fn is_empty(&self) -> bool {
            self.len() == 0
        }
    }

    // commit is a list of vals that signed.
    // use None if the val didn't sign.
    #[derive(Clone, Debug, Serialize)]
    struct MockCommit {
        hash: Hash,
        vals: Vec<Option<usize>>,
    }

    impl MockCommit {
        fn new(hash: Hash, vals: Vec<Option<usize>>) -> Self {
            Self { hash, vals }
        }
    }

    impl Commit for MockCommit {
        type ValidatorSet = MockValSet;

        fn header_hash(&self) -> Hash {
            self.hash
        }

        // just the intersection
        fn voting_power_in(&self, vals: &Self::ValidatorSet) -> Result<u64, Error> {
            let mut power = 0;

            // we only care about the Somes.
            // if there's a signer thats not in the val set,
            // we can't detect it...
            for signer_opt in &self.vals {
                if let Some(signer) = signer_opt {
                    for val in &vals.vals {
                        if signer == val {
                            power += 1
                        }
                    }
                }
            }
            Ok(power)
        }

        fn votes_len(&self) -> usize {
            self.vals.len()
        }
    }

    #[derive(Clone)]
    struct MockSignedHeader {
        header: MockHeader,
        commit: MockCommit,
    }

    impl MockSignedHeader {
        const fn new(header: MockHeader, commit: MockCommit) -> Self {
            Self { header, commit }
        }
    }

    impl SignedHeader for MockSignedHeader {
        type Header = MockHeader;
        type Commit = MockCommit;
        fn header(&self) -> &Self::Header {
            &self.header
        }
        fn commit(&self) -> &Self::Commit {
            &self.commit
        }
    }

    #[derive(Clone)]
    struct MockState {
        header: MockSignedHeader,
        vals: MockValSet,
    }

    impl TrustedState for MockState {
        type LastHeader = MockSignedHeader;
        type ValidatorSet = MockValSet;

        // XXX: how to do this without cloning?!
        fn new(header: &Self::LastHeader, vals: &Self::ValidatorSet) -> Self {
            Self {
                header: header.clone(),
                vals: vals.clone(),
            }
        }
        fn last_header(&self) -> &Self::LastHeader {
            &self.header
        }
        fn validators(&self) -> &Self::ValidatorSet {
            &self.vals
        }
    }

    // XXX: Can we do without this mock and global since we have a default impl?
    struct MockThreshold {}
    impl TrustThreshold for MockThreshold {}
    static THRESHOLD: &MockThreshold = &MockThreshold {};

    // start all blockchains from here ...
    const fn init_time() -> SystemTime {
        SystemTime::UNIX_EPOCH
    }

    // create an initial trusted state from the given vals
    fn init_state(vals_vec: Vec<usize>) -> MockState {
        let time = init_time();
        let height = 1;
        let vals = &MockValSet::new(vals_vec.clone());
        let header = MockHeader::new(height, time, vals.hash(), vals.hash());
        let commit = MockCommit::new(header.hash(), vals_vec.into_iter().map(Some).collect());
        let sh = &MockSignedHeader::new(header, commit);
        MockState::new(sh, vals)
    }

    // create the next state with the given vals and commit.
    fn next_state(
        vals_vec: Vec<usize>,
        commit_vec: Vec<Option<usize>>,
    ) -> (MockSignedHeader, MockValSet, MockValSet) {
        let time = init_time() + Duration::new(10, 0);
        let height = 10;
        let vals = MockValSet::new(vals_vec);
        let next_vals = vals.clone();
        let header = MockHeader::new(height, time, vals.hash(), next_vals.hash());
        let commit = MockCommit::new(header.hash(), commit_vec);
        (MockSignedHeader::new(header, commit), vals, next_vals)
    }

    // make a state with the given vals and commit and ensure we get the error.
    fn assert_err(ts: &MockState, vals: Vec<usize>, commit: Vec<Option<usize>>, err: Error) {
        let (un_sh, un_vals, un_next_vals) = next_state(vals, commit);
        let result = verify_single(ts, &un_sh, &un_vals, &un_next_vals, THRESHOLD);
        assert_eq!(result, Err(err));
    }

    // make a state with the given vals and commit and ensure we get no error.
    fn assert_ok(ts: &MockState, vals: Vec<usize>, commit: Vec<Option<usize>>) {
        let (un_sh, un_vals, un_next_vals) = next_state(vals, commit);
        assert!(verify_single(ts, &un_sh, &un_vals, &un_next_vals, THRESHOLD).is_ok());
    }

    // convenience vars for validators that signed commit
    static S0: Option<usize> = Some(0);
    static S1: Option<usize> = Some(1);
    static S2: Option<usize> = Some(2);
    static S3: Option<usize> = Some(3);
    static S4: Option<usize> = Some(4);
    static S5: Option<usize> = Some(5);

    // valid to skip, but invalid commit. 1 validator.
    #[test]
    fn test_verify_single_skip_1_val_verify() {
        let ts = &init_state(vec![0]);

        // 100% overlap, but wrong commit.
        // NOTE: This should be an invalid commit error since there's
        // a vote from a validator not in the set !
        // but voting_power_in isn't smart enough to see this ...
        assert_err(ts, vec![1], vec![S0], Error::InsufficientVotingPower);
    }

    // valid commit and data, starting with 1 validator.
    // test if we can skip to it.
    #[test]
    fn test_verify_single_skip_1_val_skip() {
        let ts = &init_state(vec![0]);
        let err = Error::InsufficientVotingPower;

        //*****
        // Ok

        // 100% overlap (original signer is present in commit)
        assert_ok(ts, vec![0], vec![S0]);
        assert_ok(ts, vec![0, 1], vec![S0, S1]);
        assert_ok(ts, vec![0, 1, 2], vec![S0, S1, S2]);
        assert_ok(ts, vec![0, 1, 2, 3], vec![S0, S1, S2, S3]);

        //*****
        // Err

        // 0% overlap - new val set without the original signer
        assert_err(ts, vec![1], vec![S1], err);

        // 0% overlap - val set contains original signer, but they didn't sign
        assert_err(ts, vec![0, 1, 2, 3], vec![None, S1, S2, S3], err);
    }

    // valid commit and data, starting with 2 validators.
    // test if we can skip to it.
    #[test]
    fn test_verify_single_skip_2_val_skip() {
        let ts = &init_state(vec![0, 1]);
        let err = Error::InsufficientVotingPower;

        //*************
        // OK

        // 100% overlap (both original signers still present)
        assert_ok(ts, vec![0, 1], vec![S0, S1]);
        assert_ok(ts, vec![0, 1, 2], vec![S0, S1, S2]);

        // 50% overlap (one original signer still present)
        assert_ok(ts, vec![0], vec![S0]);
        assert_ok(ts, vec![0, 1, 2, 3], vec![None, S1, S2, S3]);

        //*************
        // Err

        // 0% overlap (neither original signer still present)
        assert_err(ts, vec![2], vec![S2], err);

        // 0% overlap (original signer is still in val set but not in commit)
        assert_err(ts, vec![0, 2, 3, 4], vec![None, S2, S3, S4], err);
    }

    // valid commit and data, starting with 3 validators.
    // test if we can skip to it.
    #[test]
    fn test_verify_single_skip_3_val_skip() {
        let ts = &init_state(vec![0, 1, 2]);
        let err = Error::InsufficientVotingPower;

        //*************
        // OK

        // 100% overlap (both original signers still present)
        assert_ok(ts, vec![0, 1, 2], vec![S0, S1, S2]);
        assert_ok(ts, vec![0, 1, 2, 3], vec![S0, S1, S2, S3]);

        // 66% overlap (two original signers still present)
        assert_ok(ts, vec![0, 1], vec![S0, S1]);
        assert_ok(ts, vec![0, 1, 2, 3], vec![None, S1, S2, S3]);

        //*************
        // Err

        // 33% overlap (one original signer still present)
        assert_err(ts, vec![0], vec![S0], err);
        assert_err(ts, vec![0, 3], vec![S0, S3], err);

        // 0% overlap (neither original signer still present)
        assert_err(ts, vec![3], vec![S2], err);

        // 0% overlap (original signer is still in val set but not in commit)
        assert_err(ts, vec![0, 3, 4, 5], vec![None, S3, S4, S5], err);
    }

    #[allow(clippy::result_unwrap_used)]
    fn fixed_hash() -> Hash {
        Hash::new(Algorithm::Sha256, &Sha256::digest(&[5])).unwrap()
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
    }
}
