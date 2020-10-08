use crate::{helpers::*, validator::generate_validators, Generator, Validator};
use gumdrop::Options;
use serde::Deserialize;
use simple_error::*;
use std::convert::TryFrom;
use std::str::FromStr;
use tendermint::{block, chain, validator, AppHash, chain::Id as ChainId, block::Height};

#[derive(Debug, Options, Deserialize, Clone)]
pub struct HeaderBuilder {
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
    pub chain_id: Option<ChainId>,
    #[options(help = "block height (default: 1)")]
    pub height: Option<Height>,
    #[options(help = "time (default: now)")]
    pub time: Option<u64>,
    #[options(help = "proposer index (default: 0)")]
    pub proposer: Option<usize>,
}

impl HeaderBuilder {
    pub fn new(validators: &[Validator]) -> Self {
        HeaderBuilder {
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
    set_option!(chain_id, ChainId, Some(chain_id));
    set_option!(height, Height);
    set_option!(time, u64);
    set_option!(proposer, usize);
}

impl std::str::FromStr for HeaderBuilder {
    type Err = SimpleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let header = match parse_as::<HeaderBuilder>(s) {
            Ok(input) => input,
            Err(_) => HeaderBuilder::new(&parse_as::<Vec<Validator>>(s)?),
        };
        Ok(header)
    }
}

impl Generator<block::Header> for HeaderBuilder {
    fn merge_with_default(self, default: Self) -> Self {
        HeaderBuilder {
            validators: self.validators.or(default.validators),
            next_validators: self.next_validators.or(default.next_validators),
            chain_id: self.chain_id.or(default.chain_id),
            height: self.height.or(default.height),
            time: self.time.or(default.time),
            proposer: self.proposer.or(default.proposer),
        }
    }

    fn generate(&self) -> Result<block::Header, SimpleError> {
        let vals = match &self.validators {
            None => bail!("validator array is missing"),
            Some(vals) => vals,
        };
        let vals = generate_validators(vals)?;
        let proposer_index = self.proposer.unwrap_or(0);
        let proposer_address = if !vals.is_empty() {
            vals[proposer_index].address
        } else {
            Validator::new("a").generate().unwrap().address
        };
        let valset = validator::Set::new(vals);
        let next_valset = match &self.next_validators {
            Some(next_vals) => validator::Set::new(generate_validators(next_vals)?),
            None => valset.clone(),
        };
        let chain_id = match chain::Id::from_str(
            self.chain_id
                .clone()
                .unwrap_or_else(|| ChainId::try_from("test-chain").unwrap())
                .as_str(),
        ) {
            Ok(id) => id,
            Err(_) => bail!("failed to construct header's chain_id"),
        };
        let time = if let Some(t) = self.time {
            get_time(t)
        } else {
            tendermint::Time::now()
        };
        let header = block::Header {
            version: block::header::Version { block: 0, app: 0 },
            chain_id,
            height: block::Height::try_from(self.height.unwrap_or_default())
                .map_err(|_| SimpleError::new("height out of bounds"))?,
            time,
            last_block_id: None,
            last_commit_hash: None,
            data_hash: None,
            validators_hash: valset.hash(),
            next_validators_hash: next_valset.hash(),
            consensus_hash: valset.hash(), // TODO: currently not clear how to produce a valid hash
            app_hash: AppHash::from_hex_upper("").unwrap(),
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
        let header1 = HeaderBuilder::new(&valset1)
            .next_validators(&valset2)
            .height(10)
            .time(now1);

        let now2 = now1 + 1;
        let header2 = HeaderBuilder::new(&valset1)
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
}
