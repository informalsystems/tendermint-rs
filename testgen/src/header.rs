use crate::fuzzer::fuzz_vector;
use crate::{fuzzer, helpers::*, validator::generate_validators, Generator, Validator};
use gumdrop::Options;
use serde::{Deserialize, Serialize};
use simple_error::*;
use std::str::FromStr;
use tendermint::{block, block::Header as TMHeader, chain, validator};

#[derive(Debug, Options, Serialize, Deserialize, Clone)]
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
    pub time: Option<u64>,
    #[options(help = "proposer index (default: 0)")]
    pub proposer: Option<usize>,
}

impl Header {
    pub fn new(validators: &[Validator]) -> Self {
        Header {
            validators: Some(validators.to_vec()),
            next_validators: None,
            chain_id: None,
            height: None,
            time: None,
            proposer: None,
        }
    }
    set_option!(validators, &[Validator], Some(validators.to_vec()));
    set_option!(
        next_validators,
        &[Validator],
        Some(next_validators.to_vec())
    );
    set_option!(chain_id, &str, Some(chain_id.to_string()));
    set_option!(height, u64);
    set_option!(time, u64);
    set_option!(proposer, usize);
}

impl std::str::FromStr for Header {
    type Err = SimpleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let header = match parse_as::<Header>(s) {
            Ok(input) => input,
            Err(_) => Header::new(&parse_as::<Vec<Validator>>(s)?),
        };
        Ok(header)
    }
}

impl Generator<TMHeader> for Header {
    fn merge_with_default(self, default: Self) -> Self {
        Header {
            validators: self.validators.or(default.validators),
            next_validators: self.next_validators.or(default.next_validators),
            chain_id: self.chain_id.or(default.chain_id),
            height: self.height.or(default.height),
            time: self.time.or(default.time),
            proposer: self.proposer.or(default.proposer),
        }
    }

    fn fuzz(&self, fuzzer: &mut impl fuzzer::Fuzzer) -> Self {
        fuzzer.next();
        let fmax = 5;
        let mut fuzz = self.clone();
        if fuzz.next_validators.is_none() {
            fuzz.next_validators = fuzz.validators.clone()
        }
        if fuzzer.is_from(1, fmax) {
            fuzz.chain_id = Some(fuzzer.get_string(0))
        }
        if fuzzer.is_from(2, fmax) {
            fuzz.height = Some(fuzzer.get_u64(0))
        }
        if fuzzer.is_from(3, fmax) {
            fuzz.time = Some(fuzzer.get_u64(0))
        }
        if fuzzer.is_from(4, fmax) {
            let mut vals = fuzz.validators.unwrap_or_default();
            fuzz_vector(fuzzer, &mut vals, Validator::new(&fuzzer.get_string(0)));
            fuzz.validators = Some(vals);
        }
        if fuzzer.is_from(5, fmax) {
            let mut vals = fuzz.next_validators.unwrap_or_default();
            fuzz_vector(fuzzer, &mut vals, Validator::new(&fuzzer.get_string(1)));
            fuzz.next_validators = Some(vals);
        }
        fuzz
    }

    fn generate_fuzz(&self, fuzzer: &mut impl fuzzer::Fuzzer) -> Result<TMHeader, SimpleError> {
        fuzzer.next();
        let version = if fuzzer.is_from(1, 1) {
            block::header::Version {
                block: fuzzer.get_u64(0),
                app: fuzzer.get_u64(1),
            }
        } else {
            block::header::Version { block: 0, app: 0 }
        };
        let chain_id = self
            .chain_id
            .clone()
            .unwrap_or_else(|| "test-chain".to_string());
        let chain_id = match chain::Id::from_str(&chain_id) {
            Ok(id) => id,
            Err(_) => bail!("failed to construct header's chain_id"),
        };
        let time = if let Some(t) = self.time {
            get_time(t)
        } else {
            tendermint::Time::now()
        };
        let vals = match &self.validators {
            None => bail!("validator array is missing"),
            Some(vals) => vals.clone(),
        };
        let next_vals = match &self.next_validators {
            None => vals.clone(),
            Some(vals) => vals.clone(),
        };
        let vals = generate_validators(&vals)?;
        let proposer_index = self.proposer.unwrap_or(0);
        let proposer_address = if !vals.is_empty() {
            vals[proposer_index].address
        } else {
            Validator::new("a").generate().unwrap().address
        };
        let valset = validator::Set::new(vals);
        let next_valset = validator::Set::new(generate_validators(&next_vals)?);
        let header = TMHeader {
            version,
            chain_id,
            height: block::Height(self.height.unwrap_or(1)),
            time,
            last_block_id: None,
            last_commit_hash: None,
            data_hash: None,
            validators_hash: valset.hash(),
            next_validators_hash: next_valset.hash(),
            consensus_hash: valset.hash(), // TODO: currently not clear how to produce a valid hash
            app_hash: vec![],
            last_results_hash: None,
            evidence_hash: None,
            proposer_address,
        };
        Ok(header)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header() {
        let valset1 = [
            Validator::new("a"),
            Validator::new("b"),
            Validator::new("c"),
        ];
        let valset2 = [
            Validator::new("b"),
            Validator::new("c"),
            Validator::new("d"),
        ];

        let now1: u64 = 100;
        let header1 = Header::new(&valset1)
            .next_validators(&valset2)
            .height(10)
            .time(now1);

        let now2 = now1 + 1;
        let header2 = Header::new(&valset1)
            .next_validators(&valset2)
            .height(10)
            .time(now2);
        assert_ne!(header1.generate(), header2.generate());

        let header2 = header2.time(now1);
        assert_eq!(header1.generate(), header2.generate());

        let header3 = header2.clone().height(11);
        assert_ne!(header1.generate(), header3.generate());

        let header3 = header2.clone().validators(&valset2);
        assert_ne!(header1.generate(), header3.generate());

        let header3 = header2.clone().next_validators(&valset1);
        assert_ne!(header1.generate(), header3.generate());

        let mut block_header = header2.generate().unwrap();

        block_header.chain_id = chain::Id::from_str("chain1").unwrap();
        let header = header2.chain_id("chain1");
        assert_eq!(header.generate().unwrap(), block_header);

        block_header.proposer_address = Validator::new("c").generate().unwrap().address;
        assert_ne!(header.generate().unwrap(), block_header);

        let header = header.proposer(1);
        assert_eq!(header.generate().unwrap(), block_header);
    }

    #[test]
    fn test_header_fuzz() {
        let mut fuzzer = fuzzer::RepeatFuzzer::new(&[0, 1, 2, 3, 4, 5, 6]);

        let valset1 = [
            Validator::new("a"),
            Validator::new("b"),
            Validator::new("c"),
        ];
        let valset2 = [
            Validator::new("b"),
            Validator::new("c"),
            Validator::new("d"),
        ];

        let now: u64 = 100;
        let header = Header::new(&valset1)
            .next_validators(&valset2)
            .height(10)
            .time(now);

        let fuzz = header.fuzz(&mut fuzzer);
        assert_ne!(header.chain_id, fuzz.chain_id);

        let fuzz = header.fuzz(&mut fuzzer);
        assert_ne!(header.height, fuzz.height);

        let fuzz = header.fuzz(&mut fuzzer);
        assert_ne!(header.time, fuzz.time);

        let fuzz = header.fuzz(&mut fuzzer);
        assert_ne!(header.validators, fuzz.validators);

        let fuzz = header.fuzz(&mut fuzzer);
        assert_ne!(header.next_validators, fuzz.next_validators);

        let orig = header.generate().unwrap();

        let fuzz = header.generate_fuzz(&mut fuzzer).unwrap();
        assert_ne!(orig.version, fuzz.version);

        let fuzz = header.generate_fuzz(&mut fuzzer).unwrap();
        assert_eq!(orig, fuzz);
    }
}
