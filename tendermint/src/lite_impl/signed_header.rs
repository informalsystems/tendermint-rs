//! Light client signed header implementation.

use crate::lite::Error;
use crate::validator::Set;
use crate::{block, hash, lite};

impl lite::SignedHeader for block::signed_header::SignedHeader {
    type Header = block::Header;
    type Commit = block::signed_header::SignedHeader;

    fn header(&self) -> &block::Header {
        &self.header
    }

    fn commit(&self) -> &Self {
        &self
    }
}

impl lite::Commit for block::signed_header::SignedHeader {
    type ValidatorSet = Set;

    fn header_hash(&self) -> hash::Hash {
        self.commit.block_id.hash
    }
    fn voting_power_in(&self, validators: &Set) -> Result<u64, Error> {
        // NOTE we don't know the validators that committed this block,
        // so we have to check for each vote if its validator is already known.
        let mut signed_power = 0u64;
        for vote_opt in &self.iter() {
            // skip absent and nil votes
            // NOTE: do we want to check the validity of votes
            // for nil ?
            // TODO: clarify this!
            let vote = match vote_opt {
                Some(v) => v,
                None => continue,
            };

            // check if this vote is from a known validator
            let val_id = vote.validator_id();
            let val = match validators.validator(val_id) {
                Some(v) => v,
                None => continue,
            };

            // check vote is valid from validator
            let sign_bytes = vote.sign_bytes();

            if !val.verify_signature(&sign_bytes, vote.signature()) {
                return Err(Error::InvalidSignature);
            }
            signed_power += val.power();
        }

        Ok(signed_power)
    }

    fn votes_len(&self) -> usize {
        self.commit.precommits.len()
    }
}
