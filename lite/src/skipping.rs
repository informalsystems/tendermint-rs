use crate::types::*;

/// SkippingVerifier tries to skip headers by verifying if
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
    /// Returns the time after which the current state expires
    /// and the verifier must be reset subjectively.
    fn expires(&self) -> Time {
        self.state.last_header.bft_time() + self.trusting_period
    }

    /// Verify takes a header, a commit for the header,
    /// and the next validator set referenced by the header.
    /// Note we do not know the correct validator set for this commit -
    /// we can only check if it was signed by enough of the validators we do know about.
    /// Returns an error if verification fails.
    fn verify<C>(self, now: Time, header: H, commit: C, next_validators: V) -> Result<(), Error>
    where
        C: Commit,
    {
        // ensure the state is not expired.
        if self.expires() < now {
            return Err(Error::Expired);
        }

        // ensure the next validators in the header matches what was supplied.
        if header.next_validators_hash() != next_validators.hash() {
            return Err(Error::InvalidNextValidators);
        }

        // ensure the commit matches the header.
        if header.hash() != commit.header_hash() {
            return Err(Error::InvalidCommitValue);
        }

        // ensure that +1/3 of the trusted validators signed correctly
        self.verify_commit(commit)
    }

    /// Check that +1/3 of the last trusted validator set signed this commit.
    fn verify_commit<C>(self, commit: C) -> Result<(), Error>
    where
        C: Commit,
    {
        let total_power = self.state.validators.total_power();
        let mut signed_power: u64 = 0;

        // NOTE we don't know the validators that committed this block,
        // so we have to check for each vote if its validator is already known.
        let commit_iter = commit.into_vec().into_iter();
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
            let val = match self.state.validators.validator(val_id) {
                Some(v) => v,
                None => continue,
            };

            // check vote is valid from validator
            if !val.verify_signature(vote.sign_bytes(), vote.signature()) {
                return Err(Error::InvalidSignature);
            }
            signed_power += val.power();
        }

        // check the signers account for +1/3 of the voting power
        if signed_power * 3 <= total_power * 1 {
            return Err(Error::InsufficientVotingPower);
        }

        Ok(())
    }
}
