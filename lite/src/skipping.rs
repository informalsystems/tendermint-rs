use crate::types::*;

/// The skipping verifier lite client
/// tries to skip headers by verifying if
/// +1/3 of the last validators it trusts has signed the new header.
struct SkippingVerifier<H, V>
where
    H: Header,
    V: ValidatorsLookup,
{
    trusting_period: Time,
    state: TrustedState<H, V>,
}

impl<H, V> SkippingVerifier<H, V>
where
    H: Header,
    V: ValidatorsLookup,
{
    /// trusted state expires after the trusting period.
    fn expires(&self) -> Time {
        self.state.header.bft_time() + self.trusting_period
    }

    /// Verify takes a header, a commit for the header,
    /// and the validators for the next commit.
    /// Note we do not know the correct validator set for this commit - we can only check if
    /// it was signed by enough of the validators we do know about.
    /// Returns an error if verification fails.
    fn verify<C>(self, now: Time, header: H, commit: C, next_validators: V) -> Result<(), Error>
    where
        C: Commit,
    {
        // check if the state expired
        if self.expires() < now {
            return Err(Error::Expired);
        }

        // next validator set to trust is correctly supplied
        if header.next_validators_hash() != next_validators.hash() {
            return Err(Error::InvalidNextValidators);
        }

        // commit is for a block with this header
        if header.hash() != commit.header_hash() {
            return Err(Error::InvalidCommitValue);
        }

        // check that +1/3 of trusted validators signed correctly
        self.verify_commit(commit)
    }

    /// Check that +1/3 of the last trusted validator set signed this commit.
    fn verify_commit<C>(self, commit: C) -> Result<(), Error>
    where
        C: Commit,
    {
        let total_power = self.state.next_validators.total_power();
        let mut signed_power: u64 = 0;

        // NOTE we don't know the validators that committed this block!
        let commit_iter = commit.into_vec().into_iter();
        for vote_opt in commit_iter {
            // skip absent and nil votes
            if let None = vote_opt {
                continue;
            }
            let vote = vote_opt.unwrap();

            // check if this vote is from a known validator
            let val_id = vote.validator_id();
            let val = match self.state.next_validators.validator(val_id) {
                Some(v) => v,
                None => continue,
            };

            // check vote is valid from validator
            if !val.verify(vote.sign_bytes(), vote.signature()) {
                return Err(Error::InvalidSignature);
            }
            signed_power += val.power();
        }
        if signed_power * 3 <= total_power * 1 {
            return Err(Error::InsufficientVotingPower);
        }

        Ok(())
    }
}
