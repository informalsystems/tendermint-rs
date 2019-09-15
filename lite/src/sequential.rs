use crate::types::*;

/// SequentialVerifier verifies all headers in order,
/// where +2/3 of the correct validator set must have signed the header.
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
    /// Returns the time after which the current state expires
    /// and the verifier must be reset subjectively.
    fn expires(&self) -> Time {
        self.state.last_header.bft_time() + self.trusting_period
    }

    /// Verify takes a header, a commit for the header,
    /// and the next validator set referenced by the header.
    /// Without knowing this next validator set, we can't
    /// really verify the next header, so we make verifying
    /// the current header conditional on receiving that validator set.
    /// We could break this up into two steps, one where we verify the header,
    /// and one where we receive and validate the next validators,
    /// but for now we combine them. Returns an error if verification fails.
    /// header - height H
    /// commit - height H
    /// next_validators - height H+1
    fn verify<C>(&self, now: Time, header: H, commit: C, next_validators: V) -> Result<(), Error>
    where
        C: Commit,
    {
        // ensure the state is not expired.
        if self.expires() < now {
            return Err(Error::Expired);
        }

        // ensure the height is strictly sequential.
        if header.height() != self.state.last_header.height() + 1 {
            return Err(Error::NonSequentialHeight);
        }

        // ensure the validators in the header matches what we expect from our state.
        if header.validators_hash() != self.state.validators.hash() {
            return Err(Error::InvalidValidators);
        }

        // ensure the next validators in the header matches what was supplied.
        if header.next_validators_hash() != next_validators.hash() {
            return Err(Error::InvalidNextValidators);
        }

        // ensure the commit matches the header.
        if header.hash() != commit.header_hash() {
            return Err(Error::InvalidCommitValue);
        }

        // ensure that +2/3 validators signed correctly
        self.verify_commit(commit)
    }

    /// Check that +2/3 of the correct and trusted validator set signed this commit.
    fn verify_commit<C>(&self, commit: C) -> Result<(), Error>
    where
        C: Commit,
    {
        let total_power = self.state.validators.total_power();
        let mut signed_power: u64 = 0;

        // NOTE that the vals and commit have a 1-to-1 correspondance.
        // This means we don't need the validator IDs or to do any lookup,
        // we can just zip the iterators.
        let vals_iter = self.state.validators.into_vec().into_iter();
        let commit_iter = commit.into_vec().into_iter();
        for (val, vote_opt) in vals_iter.zip(commit_iter) {
            // skip absent and nil votes
            // NOTE: do we want to check the validity of votes
            // for nil ?
            let vote = match vote_opt {
                Some(v) => v,
                None => continue,
            };

            // check vote is valid from validator
            if !val.verify_signature(vote.sign_bytes(), vote.signature()) {
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
}
