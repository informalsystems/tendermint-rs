//! [`lite::SignedHeader`] implementation for [`block::signed_header::SignedHeader`].

use crate::lite::error::{Error, Kind};
use crate::validator::Set;
use crate::{block, hash, lite, vote};
use anomaly::fail;
use std::convert::TryFrom;

impl lite::Commit for block::signed_header::SignedHeader {
    type ValidatorSet = Set;

    fn header_hash(&self) -> hash::Hash {
        self.commit.block_id.hash
    }
    fn voting_power_in(&self, validators: &Set) -> Result<u64, Error> {
        // NOTE we don't know the validators that committed this block,
        // so we have to check for each vote if its validator is already known.
        let mut signed_power = 0u64;
        for vote in &self.signed_votes() {
            // Only count if this vote is from a known validator.
            // TODO: we still need to check that we didn't see a vote from this validator twice ...
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
        // TODO: self.commit.block_id cannot be zero in the same way as in go
        // clarify if this another encoding related issue
        if self.commit.signatures.len() == 0 {
            fail!(Kind::ImplementationSpecific, "no signatures for commit");
        }
        if self.commit.signatures.len() != vals.validators().len() {
            fail!(
                Kind::ImplementationSpecific,
                "pre-commit length: {} doesn't match validator length: {}",
                self.commit.signatures.len(),
                vals.validators().len()
            );
        }

        for commit_sig in self.commit.signatures.iter() {
            commit_sig.validate(vals)?;
        }

        Ok(())
    }
}

// this private helper function does *not* do any validation but extracts
// all non-BlockIDFlagAbsent votes from the commit:
fn non_absent_votes(commit: &block::Commit) -> Vec<vote::Vote> {
    let mut votes: Vec<vote::Vote> = Default::default();
    for (i, commit_sig) in commit.signatures.iter().enumerate() {
        if commit_sig.is_absent() {
            continue;
        }

        if let Some(val_addr) = commit_sig.validator_address {
            if let Some(sig) = commit_sig.signature.clone() {
                let vote = vote::Vote {
                    vote_type: vote::Type::Precommit,
                    height: commit.height,
                    round: commit.round,
                    block_id: Option::from(commit.block_id.clone()),
                    timestamp: commit_sig.timestamp,
                    validator_address: val_addr,
                    validator_index: u64::try_from(i)
                        .expect("usize to u64 conversion failed for validator index"),
                    signature: sig,
                };
                votes.push(vote);
            }
        }
    }
    votes
}

impl block::signed_header::SignedHeader {
    /// This is a private helper method to iterate over the underlying
    /// votes to compute the voting power (see `voting_power_in` below).
    pub fn signed_votes(&self) -> Vec<vote::SignedVote> {
        let chain_id = self.header.chain_id.to_string();
        let mut votes = non_absent_votes(&self.commit);
        votes
            .drain(..)
            .map(|vote| {
                vote::SignedVote::new(
                    (&vote).into(),
                    &chain_id,
                    vote.validator_address,
                    vote.signature,
                )
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
