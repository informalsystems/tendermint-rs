use std::str::FromStr;

use gumdrop::Options;
use serde::Deserialize;
use simple_error::*;

use tendermint::block::header::Version;
use tendermint::lite::ValidatorSet;
use tendermint::{block, chain, validator, Time};

use crate::helpers::*;
use crate::producer::Producer;
use crate::validator::{produce_validators, Validator};

#[derive(Debug, Options, Deserialize, Clone)]
pub struct Header {
    #[options(
        help = "validators (required), encoded as array of 'validator' parameters",
        parse(try_from_str = "parse_as::<Vec<Validator>>")
    )]
    pub validators: Option<Vec<Validator>>,
    #[options(
        help = "next validators (default: same as validators), encoded as array of 'validator' parameters",
        parse(try_from_str = "parse_as::<Vec<Validator>>")
    )]
    pub next_validators: Option<Vec<Validator>>,
    #[options(help = "block height (default: 1)")]
    pub height: Option<u64>,
    #[options(help = "time (default: now)")]
    pub time: Option<Time>,
}

impl Header {
    pub fn new(validators: &[Validator]) -> Self {
        Header {
            validators: Some(validators.to_vec()),
            next_validators: None,
            height: None,
            time: None,
        }
    }
    pub fn next_validators(mut self, vals: &[Validator]) -> Self {
        self.next_validators = Some(vals.to_vec());
        self
    }
    pub fn height(mut self, height: u64) -> Self {
        self.height = Some(height);
        self
    }
    pub fn time(mut self, time: Time) -> Self {
        self.time = Some(time);
        self
    }
}

impl std::str::FromStr for Header {
    type Err = SimpleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let header = match parse_as::<Header>(s) {
            Ok(input) => input,
            Err(_) => Header {
                validators: Some(parse_as::<Vec<Validator>>(s)?),
                next_validators: None,
                height: None,
                time: None,
            },
        };
        Ok(header)
    }
}


impl Producer<block::Header> for Header {
    fn parse_stdin() -> Result<Self, SimpleError> {
        let header = match parse_stdin_as::<Header>() {
            Ok(input) => input,
            Err(input) => Header {
                validators: match parse_as::<Vec<Validator>>(input.as_str()) {
                    Ok(vals) => Some(vals),
                    Err(e) => bail!("failed to read header from input: {}", e),
                },
                next_validators: None,
                height: None,
                time: None,
            },
        };
        Ok(header)
    }

    fn merge_with_default(&self, other: &Self) -> Self {
        Header {
            validators: choose_from(&self.validators, &other.validators),
            next_validators: choose_from(&self.next_validators, &other.next_validators),
            height: choose_from(&self.height, &other.height),
            time: choose_from(&self.time, &other.time),
        }
    }

    fn produce(&self) -> Result<block::Header, SimpleError> {
        if self.validators.is_none() {
            bail!("validator array is missing")
        }
        let vals = produce_validators(&self.validators.as_ref().unwrap())?;
        let valset = validator::Set::new(vals.clone());
        let next_valset = match &self.next_validators {
            Some(next_vals) => validator::Set::new(produce_validators(next_vals)?),
            None => valset.clone(),
        };
        let header = block::Header {
            version: Version { block: 0, app: 0 },
            chain_id: chain::Id::from_str("test-chain-01").unwrap(),
            height: block::Height(choose_or(self.height, 1)),
            time: choose_or(self.time, Time::now()),
            last_block_id: None,
            last_commit_hash: None,
            data_hash: None,
            validators_hash: valset.hash(),
            next_validators_hash: next_valset.hash(), // hasher.hash_validator_set(&next_valset), // next_valset.hash(),
            consensus_hash: valset.hash(), //hasher.hash_validator_set(&valset), // TODO: currently not clear how to produce a valid hash
            app_hash: vec![],
            last_results_hash: None,
            evidence_hash: None,
            proposer_address: vals[0].address,
        };
        Ok(header)
    }
}
