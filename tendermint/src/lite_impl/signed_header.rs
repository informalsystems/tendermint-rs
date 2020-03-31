//! [`lite::SignedHeader`] implementation for [`block::signed_header::SignedHeader`].

use crate::lite::error::{Error, Kind};
use crate::lite::ValidatorSet;
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
        for vote in &self.iter().unwrap() {
            // skip absent and nil votes
            // NOTE: do we want to check the validity of votes
            // for nil ?
            // TODO: clarify this!

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
        if self.commit.signatures.len() != vals.validators().len() {
            fail!(
                Kind::ImplementationSpecific,
                "pre-commit length: {} doesn't match validator length: {}",
                self.commit.signatures.len(),
                vals.validators().len()
            );
        }

        for commit_sig in self.commit.signatures.iter() {
            // returns FaultyFullNode error if it detects a signer that is not present in the validator set
            if let Some(val_addr) = commit_sig.validator_address {
                if vals.validator(val_addr) == None {
                    fail!(Kind::FaultyFullNode, reason);
                }
            }
        }

        Ok(())
    }
}

fn commit_to_votes(commit: block::Commit) -> Result<Vec<vote::Vote>, Error> {
    let mut votes: Vec<vote::Vote> = Default::default();
    for (i, commit_sig) in commit.signatures.iter().enumerate() {
        if commit_sig.is_absent() {
            continue;
        }

        match commit_sig.validator_address {
            Some(val_addr) => {
                if let Some(sig) = commit_sig.signature.clone() {
                    let vote = vote::Vote {
                        vote_type: vote::Type::Precommit,
                        height: commit.height,
                        round: commit.round,
                        block_id: Option::from(commit.block_id.clone()),
                        timestamp: commit_sig.timestamp,
                        validator_address: val_addr,
                        validator_index: i as u64,
                        signature: sig,
                    };
                    votes.push(vote);
                }
            }
            None => {
                fail!(
                    Kind::ImplementationSpecific,
                    "validator address missing in commit_sig {:#?}",
                    commit_sig
                );
            }
        }
    }
    Ok(votes)
}

impl block::signed_header::SignedHeader {
    /// This is a private helper method to iterate over the underlying
    /// votes to compute the voting power (see `voting_power_in` below).
    fn iter(&self) -> Result<Vec<vote::SignedVote>, Error> {
        let chain_id = self.header.chain_id.to_string();
        // if let Ok(mut votes) = commit_to_votes(self.commit.clone())
        let mut votes = match commit_to_votes(self.commit.clone()) {
            Ok(votes_vec) => votes_vec,
            Err(e) => return Err(e),
        };
        Ok(votes
            .drain(..)
            .map(|vote| {
                vote::SignedVote::new(
                    (&vote).into(),
                    &chain_id,
                    vote.validator_address,
                    vote.signature,
                )
            })
            .collect())
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
