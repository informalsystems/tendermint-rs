use crate::{helpers::*, Generator, Header, Validator};
use gumdrop::Options;
use serde::{Deserialize, Serialize};
use simple_error::*;
use std::convert::TryFrom;
use tendermint::{
    block::{self, parts::Header as PartSetHeader},
    signature::{Ed25519Signature, Signature, Signer},
    vote,
    vote::ValidatorIndex,
};

#[derive(Debug, Options, Serialize, Deserialize, Clone)]
pub struct Vote {
    #[options(
        help = "validator of this vote (required; can be passed via STDIN)",
        parse(try_from_str = "parse_as::<Validator>")
    )]
    pub validator: Option<Validator>,
    #[options(help = "validator index (default: from commit header)")]
    pub index: Option<u16>,
    #[options(help = "header to sign (default: commit header)")]
    pub header: Option<Header>,
    #[options(help = "vote type; 'prevote' if set, otherwise 'precommit' (default)")]
    pub prevote: Option<()>,
    #[options(help = "block height (default: from header)")]
    pub height: Option<u64>,
    #[options(help = "time (default: from header)")]
    pub time: Option<u64>,
    #[options(help = "commit round (default: from commit)")]
    pub round: Option<u32>,
    #[options(
        help = "to indicate if the vote is nil; produces a 'BlockIdFlagNil' if set, otherwise 'BlockIdFlagCommit' (default)"
    )]
    pub nil: Option<()>,
}

impl Vote {
    pub fn new(validator: Validator, header: Header) -> Self {
        Vote {
            validator: Some(validator),
            index: None,
            header: Some(header),
            prevote: None,
            height: None,
            time: None,
            round: None,
            nil: None,
        }
    }
    set_option!(index, u16);
    set_option!(header, Header);
    set_option!(prevote, bool, if prevote { Some(()) } else { None });
    set_option!(height, u64);
    set_option!(time, u64);
    set_option!(round, u32);
    set_option!(nil, bool, if nil { Some(()) } else { None });
}

impl std::str::FromStr for Vote {
    type Err = SimpleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_as::<Vote>(s)
    }
}

impl Generator<vote::Vote> for Vote {
    fn merge_with_default(self, default: Self) -> Self {
        Vote {
            validator: self.validator.or(default.validator),
            index: self.index.or(default.index),
            header: self.header.or(default.header),
            prevote: self.prevote.or(default.prevote),
            height: self.height.or(default.height),
            time: self.time.or(default.time),
            round: self.round.or(default.round),
            nil: self.nil.or(default.nil),
        }
    }

    fn generate(&self) -> Result<vote::Vote, SimpleError> {
        let validator = match &self.validator {
            None => bail!("failed to generate vote: validator is missing"),
            Some(v) => v,
        };
        let header = match &self.header {
            None => bail!("failed to generate vote: header is missing"),
            Some(h) => h,
        };
        let signer = validator.get_private_key()?;
        let block_validator = validator.generate()?;
        let block_header = header.generate()?;
        let block_id = if self.nil.is_some() {
            None
        } else {
            Some(block::Id {
                hash: block_header.hash(),
                part_set_header: PartSetHeader::new(1, block_header.hash()).unwrap(),
            })
        };
        let validator_index = match self.index {
            Some(i) => i,
            None => {
                let position = header
                    .validators
                    .as_ref()
                    .unwrap()
                    .iter()
                    .position(|v| *v == *validator);
                match position {
                    Some(i) => i as u16, // Todo: possible overflow
                    None => 0,           // we allow non-present validators for testing purposes
                }
            }
        };
        let timestamp = if let Some(t) = self.time {
            get_time(t)?
        } else {
            block_header.time
        };
        let mut vote = vote::Vote {
            vote_type: if self.prevote.is_some() {
                vote::Type::Prevote
            } else {
                vote::Type::Precommit
            },
            height: block_header.height,
            round: block::Round::try_from(self.round.unwrap_or(1)).unwrap(),
            block_id,
            timestamp: Some(timestamp),
            validator_address: block_validator.address,
            validator_index: ValidatorIndex::try_from(validator_index as u32).unwrap(),
            signature: Signature::new(vec![0_u8; Ed25519Signature::BYTE_SIZE])
                .map_err(|e| SimpleError::new(e.to_string()))?,
        };

        let sign_bytes = get_vote_sign_bytes(block_header.chain_id, &vote);
        vote.signature = Some(signer.sign(sign_bytes.as_slice()).into());

        Ok(vote)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Time;

    #[test]
    fn test_vote() {
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

        let now = Time::new(10).generate().unwrap();
        let header = Header::new(&valset1)
            .next_validators(&valset2)
            .height(10)
            .time(tendermint::Time::from_unix_timestamp(10, 0).unwrap());

        let val = &valset1[1];
        let vote = Vote::new(val.clone(), header.clone()).round(2);

        let block_val = val.generate().unwrap();
        let block_header = header.generate().unwrap();
        let block_vote = vote.generate().unwrap();

        assert_eq!(block_vote.validator_address, block_val.address);
        assert_eq!(block_vote.height, block_header.height);
        assert_eq!(block_vote.round.value(), 2);
        assert_eq!(block_vote.timestamp.unwrap(), now);
        assert_eq!(block_vote.validator_index.value(), 1);
        assert_eq!(block_vote.vote_type, vote::Type::Precommit);

        let sign_bytes = get_vote_sign_bytes(block_header.chain_id, &block_vote);
        assert!(!verify_signature(
            &valset1[0].get_public_key().unwrap(),
            &sign_bytes,
            block_vote.signature.as_ref().unwrap()
        ));
        assert!(verify_signature(
            &valset1[1].get_public_key().unwrap(),
            &sign_bytes,
            block_vote.signature.as_ref().unwrap()
        ));
    }
}
