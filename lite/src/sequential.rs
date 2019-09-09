use crate::types::*;

/// The sequentially verifying lite client
/// verifies all headers in order, where +2/3 of the correct
/// validator set must have signed the header.
struct SequentialVerifier<H, V>
where
    H: Header,
    V: Validators,
{
    trusting_period: Time,
    state: TrustedState<H, V>,
}

impl<H, V> SequentialVerifier<H, V>
where
    H: Header,
    V: Validators,
{
    /// trusted state expires after the trusting period.
    fn expires(&self) -> Time {
        self.state.header.bft_time() + self.trusting_period
    }

    /// Verify takes a header, a commit for the header, and the next validator set referenced by
    /// the header. Without knowing this next validator set, we can't really verify the next
    /// header, so we make verifying this header conditional on receiving that validator set.
    /// Returns the new TrustedState if verification passes.
    fn verify<C>(
        &self,
        now: Time,
        header: H,
        commit: C,
        next_validators: V,
    ) -> Result<(), Error> 
    where
        C: Commit,
    {
        // check if the state expired
        if self.expires() < now {
            return Err(Error::Expired);
        }

        // sequeuntial height only
        if header.height() != self.state.header.height() + 1 {
            return Err(Error::NonSequentialHeight);
        }

        // validator set for this header is already trusted
        if header.validators_hash() != self.state.next_validators.hash() {
            return Err(Error::InvalidValidators);
        }

        // next validator set to trust is correctly supplied
        if header.next_validators_hash() != next_validators.hash() {
            return Err(Error::InvalidNextValidators);
        }

        // commit is for a block with this header
        if header.hash() != commit.header_hash() {
            return Err(Error::InvalidCommitValue);
        }

        // check that +2/3 validators signed correctly
        self.verify_commit(commit)
    }

    /// Check that +2/3 of the trusted validator set signed this commit.
    fn verify_commit<C>(&self, commit: C) -> Result<(), Error>
    where
        C: Commit,
    {
        let mut signed_power: u64 = 0;
        let mut total_power: u64 = 0;

        let vals_iter = self.state.next_validators.into_vec().into_iter();
        let commit_iter = commit.into_vec().into_iter();

        for (val, vote_opt) in vals_iter.zip(commit_iter) {
            total_power += val.power();

            // skip absent and nil votes
            let vote = match vote_opt {
                Some(v) => v,
                None => continue,
            };

            // check vote is valid from validator
            if !val.verify(vote.sign_bytes(), vote.signature()) {
                return Err(Error::InvalidSignature);
            }
            signed_power += val.power();
        }

        if signed_power * 3 <= total_power * 2 {
            return Err(Error::InsufficientVotingPower)
        }

        Ok(())
    }
}
