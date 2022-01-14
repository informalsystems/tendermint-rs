use gumdrop::Options;
use serde::{Deserialize, Serialize};
use simple_error::*;
use std::collections::BTreeSet;
use std::iter::FromIterator;
use tendermint::block::{self, parts::Header as PartSetHeader, Round};

use crate::validator::sort_validators;
use crate::{helpers::*, Generator, Header, Validator, Vote};
use std::convert::TryFrom;

#[derive(Debug, Options, Serialize, Deserialize, Clone)]
pub struct Commit {
    #[options(help = "header (required)", parse(try_from_str = "parse_as::<Header>"))]
    pub header: Option<Header>,
    #[options(
        help = "votes in this commit (default: from header)",
        parse(try_from_str = "parse_as::<Vec<Vote>>")
    )]
    pub votes: Option<Vec<Vote>>,
    #[options(help = "commit round (default: 1)")]
    pub round: Option<u32>,
}

impl Commit {
    /// Make a new commit using default votes produced from the header.
    pub fn new(header: Header, round: u32) -> Self {
        let commit = Commit {
            header: Some(header),
            round: Some(round),
            votes: None,
        };
        commit.generate_default_votes()
    }
    /// Make a new commit using explicit votes.
    pub fn new_with_votes(header: Header, round: u32, votes: Vec<Vote>) -> Self {
        Commit {
            header: Some(header),
            round: Some(round),
            votes: Some(votes),
        }
    }
    set_option!(header, Header);
    set_option!(votes, Vec<Vote>);
    set_option!(round, u32);

    /// Generate commit votes from all validators in the header.
    /// This function will panic if the header is not present
    pub fn generate_default_votes(mut self) -> Self {
        let header = self.header.as_ref().unwrap();
        let val_to_vote = |(i, v): (usize, &Validator)| -> Vote {
            Vote::new(v.clone(), header.clone())
                .index(i as u16)
                .round(self.round.unwrap_or(1))
        };
        let votes = header
            .validators
            .as_ref()
            .unwrap()
            .iter()
            .enumerate()
            .map(val_to_vote)
            .collect();
        self.votes = Some(votes);
        self
    }

    /// Get a mutable reference to the vote of the given validator.
    /// This function will panic if the votes or the validator vote is not present
    pub fn vote_of_validator(&mut self, id: &str) -> &mut Vote {
        self.votes
            .as_mut()
            .unwrap()
            .iter_mut()
            .find(|v| *v.validator.as_ref().unwrap() == Validator::new(id))
            .unwrap()
    }

    /// Get a mutable reference to the vote at the given index
    /// This function will panic if the votes or the vote at index is not present
    pub fn vote_at_index(&mut self, index: usize) -> &mut Vote {
        self.votes.as_mut().unwrap().get_mut(index).unwrap()
    }
}

impl std::str::FromStr for Commit {
    type Err = SimpleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let commit = match parse_as::<Commit>(s) {
            Ok(input) => input,
            Err(_) => Commit::new(parse_as::<Header>(s)?, 1),
        };
        Ok(commit)
    }
}

impl Generator<block::Commit> for Commit {
    fn merge_with_default(self, other: Self) -> Self {
        Commit {
            header: self.header.or(other.header),
            round: self.round.or(other.round),
            votes: self.votes.or(other.votes),
        }
    }

    fn generate(&self) -> Result<block::Commit, SimpleError> {
        let header = match &self.header {
            None => bail!("failed to generate commit: header is missing"),
            Some(h) => h,
        };
        let block_header = header.generate()?;
        let block_id = block::Id {
            hash: block_header.hash(),
            part_set_header: PartSetHeader::new(1, block_header.hash()).unwrap(),
        };
        let votes = match &self.votes {
            None => self.clone().generate_default_votes().votes.unwrap(),
            Some(vs) => vs.to_vec(),
        };

        let all_vals = header.validators.as_ref().unwrap();
        let mut all_vals: BTreeSet<&Validator> = BTreeSet::from_iter(all_vals);
        let votes_vals: Vec<Validator> =
            votes.iter().map(|v| v.validator.clone().unwrap()).collect();
        all_vals.append(&mut BTreeSet::from_iter(&votes_vals));
        let all_vals: Vec<Validator> = all_vals.iter().map(|&x| x.clone()).collect();
        let all_vals = sort_validators(&all_vals);

        let vote_to_sig = |v: &Vote| -> Result<block::CommitSig, SimpleError> {
            let vote = v.generate()?;
            if vote.block_id == None {
                Ok(block::CommitSig::BlockIdFlagNil {
                    validator_address: vote.validator_address,
                    timestamp: vote.timestamp.unwrap(),
                    signature: vote.signature,
                })
            } else {
                Ok(block::CommitSig::BlockIdFlagCommit {
                    validator_address: vote.validator_address,
                    timestamp: vote.timestamp.unwrap(),
                    signature: vote.signature,
                })
            }
        };
        let val_to_sig = |val: &Validator| -> Result<block::CommitSig, SimpleError> {
            let vote = votes
                .iter()
                .find(|&vote| vote.validator.as_ref().unwrap() == val);
            match vote {
                Some(vote) => vote_to_sig(vote),
                None => Ok(block::CommitSig::BlockIdFlagAbsent),
            }
        };
        let sigs = all_vals
            .iter()
            .map(val_to_sig)
            .collect::<Result<Vec<block::CommitSig>, SimpleError>>()?;
        let commit = block::Commit {
            height: block_header.height,
            round: Round::try_from(self.round.unwrap_or(1)).unwrap(),
            block_id, /* TODO do we need at least one part?
                       * //block::Id::new(hasher.hash_header(&block_header), None), // */
            signatures: sigs,
        };
        Ok(commit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tendermint::Time;

    #[test]
    fn test_commit() {
        let valset1 = sort_validators(&[
            Validator::new("a"),
            Validator::new("b"),
            Validator::new("c"),
        ]);
        let valset2 = sort_validators(&[
            Validator::new("d"),
            Validator::new("e"),
            Validator::new("f"),
        ]);

        let header = Header::new(&valset1)
            .next_validators(&valset2)
            .height(10)
            .time(Time::from_unix_timestamp(11, 0).unwrap());

        let commit = Commit::new(header.clone(), 3);

        let block_header = header.generate().unwrap();
        let block_commit = commit.generate().unwrap();

        assert_eq!(block_commit.round.value(), 3);
        assert_eq!(block_commit.height, block_header.height);

        let mut commit = commit;
        assert_eq!(commit.vote_at_index(1).round, Some(3));
        assert_eq!(commit.vote_of_validator("b").index, Some(0));

        let votes = commit.votes.as_ref().unwrap();

        for (i, sig) in block_commit.signatures.iter().enumerate() {
            match sig {
                block::CommitSig::BlockIdFlagCommit {
                    validator_address: _,
                    timestamp: _,
                    signature,
                } => {
                    let block_vote = votes[i].generate().unwrap();
                    let sign_bytes =
                        get_vote_sign_bytes(block_header.chain_id.clone(), &block_vote);
                    assert!(!verify_signature(
                        &valset2[i].get_public_key().unwrap(),
                        &sign_bytes,
                        signature.as_ref().unwrap()
                    ));
                    assert!(verify_signature(
                        &valset1[i].get_public_key().unwrap(),
                        &sign_bytes,
                        signature.as_ref().unwrap()
                    ));
                }
                _ => panic!("signature was not a commit"),
            };
        }
    }
}
