use gumdrop::Options;
use serde::Deserialize;
use signatory::ed25519;
use signatory::public_key::PublicKeyed;
use signatory_dalek::Ed25519Signer;
use simple_error::*;

use tendermint::account;
use tendermint::public_key::PublicKey;
use tendermint::validator::{Info, ProposerPriority};
use tendermint::vote::Power;

use crate::helpers::*;
use crate::producer::Producer;

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
    pub fn voting_power(mut self, power: u64) -> Self {
        self.voting_power = Some(power);
        self
    }
    pub fn proposer_priority(mut self, priority: i64) -> Self {
        self.proposer_priority = Some(priority);
        self
    }
    pub fn signer(&self) -> Result<Ed25519Signer, SimpleError> {
        if self.id.is_none() {
            bail!("validator identifier is missing")
        }
        let mut bytes = self.id.clone().unwrap().into_bytes();
        if bytes.len() > 32 {
            bail!("identifier is too long")
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
            Err(_) => Validator {
                id: if s.is_empty() {
                    bail!("failed to parse validator")
                } else {
                    Some(s.to_string())
                },
                voting_power: None,
                proposer_priority: None,
            }
        };
        Ok(validator)
    }
}

impl Producer<Info> for Validator {
    fn merge_with_default(&self, other: &Self) -> Self {
        Validator {
            id: choose_from(&self.id, &other.id),
            voting_power: choose_from(&self.voting_power, &other.voting_power),
            proposer_priority: choose_from(&self.proposer_priority, &other.proposer_priority),
        }
    }

    fn produce(&self) -> Result<Info, SimpleError> {
        let signer = self.signer()?;
        let pk = try_with!(signer.public_key(), "failed to get a public key");
        let info = Info {
            address: account::Id::from(pk),
            pub_key: PublicKey::from(pk),
            voting_power: Power::new(choose_or(self.voting_power, 0)),
            proposer_priority: match self.proposer_priority {
                None => None,
                Some(p) => Some(ProposerPriority::new(p)),
            },
        };
        Ok(info)
    }
}

pub fn produce_validators(vals: &[Validator]) -> Result<Vec<Info>, SimpleError> {
    Ok(vals.iter().map(|v| v.produce().unwrap()).collect())
}
