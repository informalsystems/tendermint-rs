use gumdrop::Options;
use serde::Deserialize;
use simple_error::*;
use signatory::{ ed25519, public_key::PublicKeyed};
use signatory_dalek::Ed25519Signer;
use tendermint::{
    account, public_key::PublicKey, validator, vote
};
use crate::{Generator, helpers::*};

#[derive(Debug, Options, Deserialize, Clone)]
pub struct Validator {
    #[options(help = "validator id (required; can be passed via STDIN)")]
    pub id: Option<String>,
    #[options(help = "voting power of this validator (default: 0)", meta = "POWER")]
    pub voting_power: Option<u64>,
    #[options(
        help = "proposer priority of this validator (default: none)",
        meta = "PRIORITY"
    )]
    pub proposer_priority: Option<i64>,
}




impl Validator {
    pub fn new(id: &str) -> Self {
        Validator {
            id: Some(id.to_string()),
            voting_power: None,
            proposer_priority: None,
        }
    }
    set_option!(voting_power, u64);
    set_option!(proposer_priority, i64);

    pub fn get_signer(&self) -> Result<Ed25519Signer, SimpleError> {
        if self.id.is_none() {
            bail!("validator identifier is missing")
        }
        let mut bytes = self.id.clone().unwrap().into_bytes();
        if bytes.is_empty() {
            bail!("empty validator identifier")
        }
        if bytes.len() > 32 {
            bail!("validator identifier is too long")
        }
        bytes.extend(vec![0u8; 32 - bytes.len()].iter());
        let seed = require_with!(
            ed25519::Seed::from_bytes(bytes),
            "failed to construct a seed"
        );
        Ok(Ed25519Signer::from(&seed))
    }
}

impl std::str::FromStr for Validator {
    type Err = SimpleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let validator = match parse_as::<Validator>(s) {
            Ok(input) => input,
            Err(_) => Validator::new(s)
        };
        Ok(validator)
    }
}

impl Generator<validator::Info> for Validator {
    fn merge_with_default(&self, default: &Self) -> Self {
        Validator {
            id: choose_from(&self.id, &default.id),
            voting_power: choose_from(&self.voting_power, &default.voting_power),
            proposer_priority: choose_from(&self.proposer_priority, &default.proposer_priority),
        }
    }

    fn generate(&self) -> Result<validator::Info, SimpleError> {
        let signer = self.get_signer()?;
        let pk = try_with!(signer.public_key(), "failed to get a public key");
        let info = validator::Info {
            address: account::Id::from(pk),
            pub_key: PublicKey::from(pk),
            voting_power: vote::Power::new(choose_or(self.voting_power, 0)),
            proposer_priority: match self.proposer_priority {
                None => None,
                Some(p) => Some(validator::ProposerPriority::new(p)),
            },
        };
        Ok(info)
    }
}

pub fn generate_validators(vals: &[Validator]) -> Result<Vec<validator::Info>, SimpleError> {
    Ok(vals.iter().map(|v| v.generate()).collect::<Result<Vec<validator::Info>, SimpleError>>()?)
}
