use gumdrop::Options;
use serde::Deserialize;
use signatory::{
    ed25519,
    signature::{Signature as _, Signer},
};
use simple_error::*;
use tendermint::{block, lite, signature::Signature, vote, Time};

use crate::{helpers::*, Generator, Header, Validator};

#[derive(Debug, Options, Deserialize, Clone)]
pub struct Vote {
    #[options(
        help = "validator of this vote (required; can be passed via STDIN)",
        parse(try_from_str = "parse_as::<Validator>")
    )]
    pub validator: Option<Validator>,
    #[options(help = "validator index (default: from commit header)")]
    pub index: Option<u64>,
    #[options(help = "header to sign (default: commit header)")]
    pub header: Option<Header>,
    #[options(help = "vote type; 'precommit' if set, otherwise 'prevote' (default)")]
    pub precommit: Option<()>,
    #[options(help = "block height (default: from header)")]
    pub height: Option<u64>,
    #[options(help = "time (default: from header)")]
    pub time: Option<Time>,
    #[options(help = "commit round (default: from commit)")]
    pub round: Option<u64>,
}

impl Vote {
    pub fn new(validator: &Validator, header: &Header) -> Self {
        Vote {
            validator: Some(validator.clone()),
            index: None,
            header: Some(header.clone()),
            precommit: None,
            height: None,
            time: None,
            round: None,
        }
    }
    set_option!(index, u64);
    set_option!(header, &Header, Some(header.clone()));
    set_option!(precommit, bool, if precommit { Some(()) } else { None });
    set_option!(height, u64);
    set_option!(time, Time);
    set_option!(round, u64);
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
            precommit: self.precommit.or(default.precommit),
            height: self.height.or(default.height),
            time: self.time.or(default.time),
            round: self.round.or(default.round),
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
        let signer = validator.get_signer()?;
        let block_validator = validator.generate()?;
        let block_header = header.generate()?;
        let block_id = block::Id::new(lite::Header::hash(&block_header), None);
        let validator_index = match self.index {
            Some(i) => i,
            None => match header.validators.as_ref().unwrap().iter().position(|v| *v == *validator) {
                Some(i) => i as u64,
                None => bail!("failed to generate vote: no index given and validator not present in the header")
            }
        };
        let mut vote = vote::Vote {
            vote_type: if self.precommit.is_some() {
                vote::Type::Precommit
            } else {
                vote::Type::Prevote
            },
            height: block_header.height,
            round: self.round.unwrap_or(1),
            block_id: Some(block_id),
            timestamp: block_header.time,
            validator_address: block_validator.address,
            validator_index,
            signature: Signature::Ed25519(try_with!(
                ed25519::Signature::from_bytes(&[0_u8; ed25519::SIGNATURE_SIZE]),
                "failed to construct empty ed25519 signature"
            )),
        };
        let sign_bytes = get_vote_sign_bytes(block_header.chain_id.as_str(), &vote);
        vote.signature = Signature::Ed25519(signer.sign(sign_bytes.as_slice()));
        Ok(vote)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let now = Time::now();
        let header = Header::new(&valset1)
            .next_validators(&valset2)
            .height(10)
            .time(now);

        let val = &valset1[1];
        let vote = Vote::new(val, &header).round(2);

        let block_val = val.generate().unwrap();
        let block_header = header.generate().unwrap();
        let block_vote = vote.generate().unwrap();

        assert_eq!(block_vote.validator_address, block_val.address);
        assert_eq!(block_vote.height, block_header.height);
        assert_eq!(block_vote.round, 2);
        assert_eq!(block_vote.timestamp, now);
        assert_eq!(block_vote.validator_index, 1);
        assert_eq!(block_vote.vote_type, vote::Type::Prevote);

        let sign_bytes = get_vote_sign_bytes(block_header.chain_id.as_str(), &block_vote);
        assert!(!verify_signature(
            &valset1[0].get_verifier().unwrap(),
            &sign_bytes,
            &block_vote.signature
        ));
        assert!(verify_signature(
            &valset1[1].get_verifier().unwrap(),
            &sign_bytes,
            &block_vote.signature
        ));
    }
}
