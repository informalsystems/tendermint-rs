use std::str::FromStr;

use gumdrop::Options;
use serde::Deserialize;
use simple_error::*;

use tendermint::block::header::Version;
use tendermint::lite::ValidatorSet;
use tendermint::{block, chain, validator, Time};

use crate::helpers::*;
use crate::generator::Generator;
use crate::validator::{generate_validators, Validator};

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
    #[options(help = "chain id (default: test-chain)")]
    pub chain_id: Option<String>,
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
            chain_id: None,
            height: None,
            time: None,
        }
    }
    gen_setter!(next_validators, &[Validator], next_validators.to_vec());
    gen_setter!(chain_id, &str, chain_id.to_string());
    gen_setter!(height, u64);
    gen_setter!(time, Time);
}

impl std::str::FromStr for Header {
    type Err = SimpleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let header = match parse_as::<Header>(s) {
            Ok(input) => input,
            Err(_) => Header {
                validators: Some(parse_as::<Vec<Validator>>(s)?),
                next_validators: None,
                chain_id: None,
                height: None,
                time: None,
            },
        };
        Ok(header)
    }
}


impl Generator<block::Header> for Header {
    fn merge_with_default(&self, default: &Self) -> Self {
        Header {
            validators: choose_from(&self.validators, &default.validators),
            next_validators: choose_from(&self.next_validators, &default.next_validators),
            chain_id: choose_from(&self.chain_id, &default.chain_id),
            height: choose_from(&self.height, &default.height),
            time: choose_from(&self.time, &default.time),
        }
    }

    fn generate(&self) -> Result<block::Header, SimpleError> {
        if self.validators.is_none() {
            bail!("validator array is missing")
        }
        let vals = generate_validators(&self.validators.as_ref().unwrap())?;
        let valset = validator::Set::new(vals.clone());
        let next_valset = match &self.next_validators {
            Some(next_vals) => validator::Set::new(generate_validators(next_vals)?),
            None => valset.clone(),
        };
        let  chain_id = match chain::Id::from_str(
            choose_or(self.chain_id.clone(), "test-chain".to_string()).as_str()) {
            Ok(id) => id,
            Err(_) => bail!("failed to construct header chain_id")
        };
        let header = block::Header {
            version: Version { block: 0, app: 0 },
            chain_id: chain_id,
            height: block::Height(choose_or(self.height, 1)),
            time: choose_or(self.time, Time::now()),
            last_block_id: None,
            last_commit_hash: None,
            data_hash: None,
            validators_hash: valset.hash(),
            next_validators_hash: next_valset.hash(),
            consensus_hash: valset.hash(), // TODO: currently not clear how to produce a valid hash
            app_hash: vec![],
            last_results_hash: None,
            evidence_hash: None,
            proposer_address: vals[0].address,
        };
        Ok(header)
    }
}
