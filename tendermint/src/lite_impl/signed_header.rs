//! [`lite::SignedHeader`] implementation for [`block::signed_header::SignedHeader`].

use crate::lite::error::{Error, Kind};
use crate::validator::Set;
use crate::{block, hash, lite, vote};
use anomaly::fail;

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
                fail!(
                    Kind::ImplementationSpecific,
                    "Couldn't verify signature {:?} with validator {:?} on sign_bytes {:?}",
                    vote.signature(),
                    val,
                    sign_bytes,
                );
            }
            signed_power += val.power();
        }

        Ok(signed_power)
    }

    fn validate(&self, vals: &Self::ValidatorSet) -> Result<(), Error> {
        if self.commit.precommits.len() != vals.validators().len() {
            fail!(
                lite::error::Kind::ImplementationSpecific,
                "pre-commit length: {} doesn't match validator length: {}",
                self.commit.precommits.len(),
                vals.validators().len()
            );
        }
        // TODO: compare to the go code for more implementation related checks and clarify if this:
        // https://github.com/interchainio/tendermint-rs/pull/143/commits/0a30022fa47e909e6c7b20417dd178c8a3b84838#r374958528
        // should go here or somewhere else

        // make sure each vote is for the correct header
        for precommit in self.commit.precommits.iter() {
            match precommit {
                Some(p) => {
                    if p.header_hash() != self.header_hash() {
                        return Err(lite::error::Kind::ImplementationSpecific
                        .context(format!(
                            "precommit.block_id doesn't match with commit.block_id"
                        ))
                        .into());
                    }

                    
                },
                None => ()

            }
            
        }
        Ok(())
    }
}

impl block::signed_header::SignedHeader {
    /// This is a private helper method to iterate over the underlying
    /// votes to compute the voting power (see `voting_power_in` below).
    fn iter(&self) -> Vec<Option<vote::SignedVote>> {
        let chain_id = self.header.chain_id.to_string();
        let mut votes = self.commit.precommits.clone().into_vec();
        votes
            .drain(..)
            .map(|opt| {
                opt.map(|vote| {
                    vote::SignedVote::new(
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

// type alias the concrete types to make the From impls more readable
type TMSignedHeader = block::signed_header::SignedHeader;
type TMBlockHeader = block::header::Header;

impl From<block::signed_header::SignedHeader>
    for lite::types::SignedHeader<TMSignedHeader, TMBlockHeader>
{
    fn from(sh: block::signed_header::SignedHeader) -> Self {
        Self::new(sh.clone(), sh.header)
    }
}

impl From<&block::signed_header::SignedHeader>
    for lite::types::SignedHeader<TMSignedHeader, TMBlockHeader>
{
    fn from(sh: &block::signed_header::SignedHeader) -> Self {
        Self::new(sh.clone(), sh.clone().header)
    }
}
