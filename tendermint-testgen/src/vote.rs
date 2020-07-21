use gumdrop::Options;
use serde::Deserialize;
use simple_error::*;
use tendermint::{
    Time, vote, block, lite, amino_types,
                 signature::Signature };
use signatory::{
    ed25519,
    signature::{ Signature as _, Signer }
};
use crate::{Generator, Validator, Header, helpers::*};

#[derive(Debug, Options, Deserialize, Clone)]
pub struct Vote {
    #[options(help = "validator of this vote (required; can be passed via STDIN)",
      parse(try_from_str = "parse_as::<Validator>"))]
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
            round: None
        }
    }
    set_option!(index, u64);
    set_option!(header, &Header, Some(header.clone()));
    set_option!(precommit, bool, if precommit {Some(())} else {None});
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
    fn merge_with_default(&self, default: &Self) -> Self {
        Vote {
            validator: choose_from(&self.validator, &default.validator),
            index: choose_from(&self.index, &default.index),
            header: choose_from(&self.header, &default.header),
            precommit: choose_from(&self.precommit, &default.precommit),
            height: choose_from(&self.height, &default.height),
            time: choose_from(&self.time, &default.time),
            round: choose_from(&self.round, &default.round)
        }
    }

    fn generate(&self) -> Result<vote::Vote, SimpleError> {
        let validator = match &self.validator {
            None => bail!("failed to generate vote: validator is missing"),
            Some(v) => v
        };
        let header = match &self.header {
            None => bail!("failed to generate vote: header is missing"),
            Some(h) => h
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
            vote_type: if self.precommit.is_some() { vote::Type::Precommit } else { vote::Type::Prevote },
            height: block_header.height,
            round: choose_or(self.round, 1),
            block_id: Some(block_id.clone()),
            timestamp: block_header.time,
            validator_address: block_validator.address,
            validator_index,
            signature: Signature::Ed25519(
                try_with!(ed25519::Signature::from_bytes(&[0_u8; ed25519::SIGNATURE_SIZE]), "failed to construct empty ed25519 signature"),
            ),
        };
        let signed_vote = vote::SignedVote::new(
            amino_types::vote::Vote::from(&vote),
            block_header.chain_id.as_str(),
            vote.validator_address,
            vote.signature
        );
        let sign_bytes = signed_vote.sign_bytes();
        vote.signature = Signature::Ed25519(try_with!(signer.try_sign(sign_bytes.as_slice()), "failed to sign using ed25519 signature"));
        Ok(vote)
    }
}
