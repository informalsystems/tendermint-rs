use gumdrop::Options;
use serde::Deserialize;
use simple_error::*;
use tendermint::{ block, lite };

use crate::{Generator, Validator, Header, Vote, helpers::*};

#[derive(Debug, Options, Deserialize)]
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
    set_option!(round, u64);

    pub fn header(&self) -> Result<&Header, SimpleError> {
        match &self.header{
            None => bail!("commit header is missing"),
            Some(h) => Ok(h)
        }
    }

    pub fn votes(&self) -> Result<&Vec<Vote>, SimpleError> {
        match &self.votes{
            None => bail!("commit votes is missing"),
            Some(vs) => Ok(vs)
        }
    }
    pub fn generate_default_votes(mut self) -> Result<(), SimpleError>  {
        let header = self.header()?;
        let val_to_vote = |(i, v): (usize, &Validator)| -> Vote {
            Vote::new(v, header)
                .index(&(i as u64))
        };
        let votes = header.validators
            .as_ref()
            .unwrap()
            .iter()
            .enumerate()
            .map(val_to_vote)
            .collect();
        self.votes = Some(votes);
        Ok(())
    }

    pub fn vote_of(&self, val: &Validator) -> Result<&Vote, SimpleError> {
        let vote = require_with!(self.votes()?.iter().find(|v| *v.validator.as_ref().unwrap() == *val), "can't find vote by validator");
        Ok(vote)
    }

    pub fn vote_at(&self, index: usize) -> Result<&Vote, SimpleError> {
        let vote = require_with!(self.votes()?.get(index), "can't find vote by index");
        Ok(vote)
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
        let header = self.header()?;
        let votes = self.votes()?;
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
