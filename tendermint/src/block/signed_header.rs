//! [`SignedHeader`] contains commit and and block header.
//! It is what the rpc endpoint /commit returns and hence can be used by a
//! light client.
use crate::{block, hash, lite, lite::Error, validator::Set, vote};
use serde::{Deserialize, Serialize};

/// Signed block headers
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignedHeader {
    /// Block header
    pub header: block::Header,
    /// Commit containing signatures for the header
    pub commit: block::Commit,
}

impl lite::SignedHeader for SignedHeader {
    type Header = block::Header;
    type Commit = SignedHeader;

    fn header(&self) -> &block::Header {
        &self.header
    }

    fn commit(&self) -> &Self {
        &self
    }
}

impl SignedHeader {
    /// This is a private helper method to iterate over the underlying
    /// votes to compute the voting power (see `voting_power_in` below).
    fn iter(&self) -> Vec<Option<vote::Signed>> {
        let chain_id = self.header.chain_id.to_string();
        let mut votes = self.commit.precommits.clone().into_vec();
        votes
            .drain(..)
            .map(|opt| {
                opt.map(|vote| {
                    vote::Signed::new(
                        (&vote).into(),
                        &chain_id,
                        vote.validator_address,
                        vote.signature,
                    )
                })
            })
            .collect()
    }
}

impl lite::Commit for SignedHeader {
    type ValidatorSet = Set;

    fn header_hash(&self) -> hash::Hash {
        self.commit.block_id.hash
    }
    fn votes_len(&self) -> usize {
        self.commit.precommits.len()
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
}
