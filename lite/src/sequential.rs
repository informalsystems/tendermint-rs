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
        self,
        now: Time,
        header: H,
        commit: C,
        next_validators: V,
    ) -> Option<TrustedState<H, V>>
    where
        C: Commit,
    {
        // check if the state expired
        if self.expires() < now {
            return None;
        }

        // sequeuntial height only
        if header.height() != self.state.header.height() + 1 {
            return None;
        }

        // validator set for this header is already trusted
        if header.validators_hash() != self.state.next_validators.hash() {
            return None;
        }

        // next validator set to trust is correctly supplied
        if header.next_validators_hash() != next_validators.hash() {
            return None;
        }

        // commit is for a block with this header
        if header.hash() != commit.header_hash() {
            return None;
        }

        // check that +2/3 validators signed correctly
        if self.verify_commit_full(commit) {
            return None;
        }

        Some(TrustedState {
            header,
            next_validators,
        })
    }

    /// Check that +2/3 of the trusted validator set signed this commit.
    fn verify_commit_full<C>(self, commit: C) -> bool
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
            if let None = vote_opt {
                continue;
            }
            let vote = vote_opt.unwrap();

            // check vote is valid from validator
            if !val.verify(vote.sign_bytes(), vote.signature()) {
                return false;
            }
            signed_power += val.power();
        }
        signed_power * 3 > total_power * 2
    }
}
