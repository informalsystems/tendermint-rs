use gumdrop::Options;
use serde::Deserialize;
use simple_error::*;
use tendermint::{ block, lite };

use crate::{Generator, Validator, Header, Vote, helpers::*};

#[derive(Debug, Options, Deserialize, Clone)]
pub struct Commit {
    #[options(help = "header (required)", parse(try_from_str = "parse_as::<Header>"))]
    pub header: Option<Header>,
    #[options(help = "votes in this commit (default: from header)",
    parse(try_from_str = "parse_as::<Vec<Vote>>"))]
    pub votes: Option<Vec<Vote>>,
    #[options(help = "commit round (default: 1)")]
    pub round: Option<u64>
}

impl Commit {
    pub fn new(header: &Header) -> Self {
        Commit {
            header: Some(header.clone()),
            round: None,
            votes: None
        }
    }
    set_option!(votes, Vec<Vote>);
    set_option!(round, u64);

    // generate commit votes from all validators in the header
    // this function will panic if the header is not present
    pub fn generate_default_votes(mut self) -> Self  {
        let header = self.header.as_ref().unwrap();
        let val_to_vote = |(i, v): (usize, &Validator)| -> Vote {
            Vote::new(v, header)
                .index(i as u64)
                .round(choose_or(self.round, 1))
        };
        let votes = header.validators
            .as_ref()
            .unwrap()
            .iter()
            .enumerate()
            .map(val_to_vote)
            .collect();
        self.votes = Some(votes);
        self
    }

    // get a mutable reference to the vote of the given validator
    // this function will panic if the votes or the validator vote is not present
    pub fn vote_of(&mut self, val: &str) -> &mut Vote {
        self.votes.as_mut().unwrap().iter_mut().find(
            |v| *v.validator.as_ref().unwrap() == Validator::new(val)
            ).unwrap()
    }

    // get a mutable reference to the vote at the given index
    // this function will panic if the votes or the vote at index is not present
    pub fn vote_at(&mut self, index: usize) -> &mut Vote {
        self.votes.as_mut().unwrap().get_mut(index).unwrap()
    }
}

impl std::str::FromStr for Commit {
    type Err = SimpleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let commit = match parse_as::<Commit>(s) {
            Ok(input) => input,
            Err(_) => Commit::new(&parse_as::<Header>(s)?)
        };
        Ok(commit)
    }
}

impl Generator<block::Commit> for Commit {
    fn merge_with_default(&self, other: &Self) -> Self {
        Commit {
            header: choose_from(&self.header, &other.header),
            round: choose_from(&self.round, &other.round),
            votes: choose_from(&self.votes, &other.votes)
        }
    }

    fn generate(&self) -> Result<block::Commit, SimpleError> {
        let header = match &self.header{
            None => bail!("failed to generate commit: header is missing"),
            Some(h) => h
        };
        let votes = match &self.votes{
            None => bail!("failed to generate commit: votes are missing"),
            Some(vs) => vs
        };
        let block_header = header.generate()?;
        let block_id = block::Id::new(lite::Header::hash(&block_header), None);

        let vote_to_sig = |v: &Vote| -> Result<block::CommitSig, SimpleError> {
            let vote = v.generate()?;
            Ok(block::CommitSig::BlockIDFlagCommit {
                validator_address: vote.validator_address,
                timestamp: vote.timestamp,
                signature: vote.signature,
            })
        };
        let sigs = votes.iter()
            .map(vote_to_sig)
            .collect::<Result<Vec<block::CommitSig>, SimpleError>>()?;
        let commit = block::Commit {
            height: block_header.height,
            round: choose_or(self.round, 1),
            block_id, // TODO do we need at least one part? //block::Id::new(hasher.hash_header(&block_header), None), //
            signatures: block::CommitSigs::new(sigs),
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
        let valset1 = [Validator::new("a"), Validator::new("b"), Validator::new("c")];
        let valset2 = [Validator::new("b"), Validator::new("c"), Validator::new("d")];

        let now = Time::now();
        let header = Header::new(&valset1).next_validators(&valset2).height(10).time(now);

        let commit = Commit::new(&header).round(3).generate_default_votes();

        let block_header = header.generate().unwrap();
        let block_commit = commit.generate().unwrap();

        assert_eq!(block_commit.round, 3);
        assert_eq!(block_commit.height, block_header.height);

        let mut commit = commit;
        assert_eq!(commit.vote_at(1).round, Some(3));
        assert_eq!(commit.vote_of("a").index, Some(0));

        let votes = commit.votes.as_ref().unwrap();

        for (i, sig) in block_commit.signatures.iter().enumerate() {
            match sig {
                block::CommitSig::BlockIDFlagCommit{
                    validator_address: _, timestamp: _, signature
                } => {
                    let block_vote = votes[i].generate().unwrap();
                    let sign_bytes = get_vote_sign_bytes(block_header.chain_id.as_str(), &block_vote);
                    assert!(!verify_signature(&valset2[i].get_verifier().unwrap(), &sign_bytes, signature));
                    assert!(verify_signature(&valset1[i].get_verifier().unwrap(), &sign_bytes, signature));
                }
                _ => assert!(false)
            };
        }
    }
}