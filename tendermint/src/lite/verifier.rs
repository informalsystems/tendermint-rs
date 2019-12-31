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

/// Validate the validators, next validators, and commit against the header.
// TODO(EB): consider making this a method on Commit so the details are hidden,
// and so we can remove the votes_len() method (that check would be part of the
// methods implementation). These checks aren't reflected
// explicitly in the spec yet, only in the sentence "Additional checks should
// be done in the implementation to ensure header is well formed".
pub fn validate_signed_header_and_vals<H, V, C>(
    header: &H,
    commit: &C,
    vals: &V,
    next_vals: &V,
) -> Result<(), Error>
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
    // NOTE: this is commit structure specifc and should be
    // hidden from the light client ...
    if vals.len() != commit.votes_len() {
        return Err(Error::InvalidCommitLength);
    }

    Ok(())
}

/// Verify that +2/3 of the correct validator set signed this commit.
/// NOTE: these validators are expected to be the correct validators for the commit,
/// but since we're using voting_power_in, we can't actually detect if there's
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
/// but there may be some intersection. The trust_level parameter allows clients to require more
/// than +1/3 by implementing the TrustLevel trait accordingly.
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

mod tests {
    use super::*;
    use crate::{hash::Algorithm, Hash};
    use serde::Serialize;
    use sha2::{Digest, Sha256};

    #[derive(Debug, Serialize)]
    struct MockHeader {
        height: u64,
        time: SystemTime,
        vals: Hash,
        next_vals: Hash,
    }

    impl MockHeader {
        fn new(height: u64, time: SystemTime, vals: Hash, next_vals: Hash) -> MockHeader {
            MockHeader {
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
            let encoded = serde_json::to_vec(self).unwrap();
            let hashed = Sha256::digest(&encoded);
            Hash::new(Algorithm::Sha256, &hashed).unwrap()
        }
    }

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
