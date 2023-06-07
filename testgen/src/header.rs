use core::time::Duration;
use std::{
    convert::{TryFrom, TryInto},
    str::FromStr,
};

use gumdrop::Options;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use simple_error::*;
use tendermint::{block, chain, validator, AppHash, Hash, Time};
use time::OffsetDateTime;

use crate::{helpers::*, validator::generate_validators, Generator, Validator};

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
    #[serde(deserialize_with = "deserialize_time")]
    #[serde(serialize_with = "serialize_time")]
    pub time: Option<Time>,
    #[options(help = "proposer index (default: 0)")]
    pub proposer: Option<usize>,
    #[options(help = "last block id hash (default: Hash::None)")]
    pub last_block_id_hash: Option<Hash>,
}

// Serialize and deserialize time only up to second precision for integration with MBT.
// This is ok as long as the serialized form is only used exclusively for MBT.
// Otherwise we will have to find other ways to serialize time at least down to
// millisecond precision, at the same time still being able to support that in MBT.
fn deserialize_time<'de, D>(deserializer: D) -> Result<Option<Time>, D::Error>
where
    D: Deserializer<'de>,
{
    let m_secs = <Option<i64>>::deserialize(deserializer)?;
    let m_time = m_secs.map(|secs| Time::from_unix_timestamp(secs, 0).unwrap());

    Ok(m_time)
}

fn serialize_time<S>(m_time: &Option<Time>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let m_secs = m_time.map(|time| {
        let datetime: OffsetDateTime = time.into();
        datetime.unix_timestamp()
    });

    m_secs.serialize(serializer)
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
            last_block_id_hash: None,
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
    set_option!(time, Time);
    set_option!(proposer, usize);
    set_option!(last_block_id_hash, Hash);

    pub fn next(&self) -> Self {
        let height = self.height.expect("Missing previous header's height");
        // if no time is found, then we simple correspond it to the header height
        let time = self
            .time
            .unwrap_or_else(|| Time::from_unix_timestamp(height.try_into().unwrap(), 0).unwrap());
        let validators = self.validators.clone().expect("Missing validators");
        let next_validators = self.next_validators.clone().unwrap_or(validators);

        let prev_header = self.generate().unwrap();
        let last_block_id_hash = prev_header.hash();

        Self {
            validators: Some(next_validators.clone()),
            next_validators: Some(next_validators),
            chain_id: self.chain_id.clone(),
            height: Some(height + 1),
            time: Some((time + Duration::from_secs(1)).unwrap()),
            proposer: self.proposer, // TODO: proposer must be incremented
            last_block_id_hash: Some(last_block_id_hash),
        }
    }
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

impl Generator<block::Header> for Header {
    fn merge_with_default(self, default: Self) -> Self {
        Header {
            validators: self.validators.or(default.validators),
            next_validators: self.next_validators.or(default.next_validators),
            chain_id: self.chain_id.or(default.chain_id),
            height: self.height.or(default.height),
            time: self.time.or(default.time),
            proposer: self.proposer.or(default.proposer),
            last_block_id_hash: self.last_block_id_hash.or(default.last_block_id_hash),
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
        let valset = validator::Set::without_proposer(vals);
        let validators_hash = valset.hash();
        let next_valset = match &self.next_validators {
            Some(next_vals) => validator::Set::without_proposer(generate_validators(next_vals)?),
            None => valset,
        };
        let chain_id = match chain::Id::from_str(
            self.chain_id
                .clone()
                .unwrap_or_else(|| "test-chain".to_string())
                .as_str(),
        ) {
            Ok(id) => id,
            Err(_) => bail!("failed to construct header's chain_id"),
        };

        let time: Time = self.time.unwrap_or_else(Time::now);

        let last_block_id = self.last_block_id_hash.map(|hash| block::Id {
            hash,
            part_set_header: Default::default(),
        });

        let header = block::Header {
            // block version in Tendermint-go is hardcoded with value 11
            // so we do the same with MBT for now for compatibility
            version: block::header::Version { block: 11, app: 0 },
            chain_id,
            height: block::Height::try_from(self.height.unwrap_or(1))
                .map_err(|_| SimpleError::new("height out of bounds"))?,
            time,
            last_block_id,
            last_commit_hash: None,
            data_hash: None,
            validators_hash,
            next_validators_hash: next_valset.hash(),
            consensus_hash: validators_hash, // TODO: currently not clear how to produce a valid hash
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
    use core::time::Duration;

    use tendermint::Time;

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

        let now1 = Time::now();
        let header1 = Header::new(&valset1)
            .next_validators(&valset2)
            .height(10)
            .time(now1);

        let now2 = (now1 + Duration::from_secs(1)).unwrap();
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
}
