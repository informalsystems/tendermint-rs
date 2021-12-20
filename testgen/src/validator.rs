use ed25519_consensus::SigningKey as Ed25519SigningKey;
use gumdrop::Options;
use serde::{Deserialize, Serialize};
use simple_error::*;
use tendermint::{
    account, consensus::state::Ordering, private_key, public_key, public_key::PublicKey, validator,
    vote,
};

use crate::{helpers::*, Generator};

#[derive(Debug, Options, Serialize, Deserialize, Clone)]
pub struct Validator {
    #[options(help = "validator id (required; can be passed via STDIN)")]
    pub id: Option<String>,
    #[options(help = "voting power of this validator (default: 0)", meta = "POWER")]
    pub voting_power: Option<u64>,
    #[options(
        help = "proposer priority of this validator (default: 0)",
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
    // Question: Why do we need this option since we're already initializing id with fn new()??
    set_option!(id, &str, Some(id.to_string()));
    set_option!(voting_power, u64);
    set_option!(proposer_priority, i64);

    /// Get private key for this validator companion.
    pub fn get_private_key(&self) -> Result<private_key::Ed25519, SimpleError> {
        let id = match &self.id {
            None => bail!("validator identifier is missing"),
            Some(id) => id,
        };
        if id.is_empty() {
            bail!("empty validator identifier")
        }
        let mut bytes = id.clone().into_bytes();
        if bytes.len() > 32 {
            bail!("validator identifier is too long")
        }
        bytes.extend(vec![0u8; 32 - bytes.len()].iter());
        let signing_key = require_with!(
            Ed25519SigningKey::try_from(&bytes[..]).ok(),
            "failed to construct a seed from validator identifier"
        );
        Ok(signing_key)
    }

    /// Get public key for this validator companion.
    pub fn get_public_key(&self) -> Result<public_key::Ed25519, SimpleError> {
        self.get_private_key()
            .map(|secret_key| secret_key.verification_key())
    }
}

impl std::str::FromStr for Validator {
    type Err = SimpleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let validator = match parse_as::<Validator>(s) {
            Ok(input) => input,
            Err(_) => Validator::new(s),
        };
        Ok(validator)
    }
}

impl std::cmp::PartialEq for Validator {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl std::cmp::Eq for Validator {}

impl std::cmp::Ord for Validator {
    fn cmp(&self, other: &Self) -> Ordering {
        self.generate()
            .unwrap()
            .address
            .cmp(&other.generate().unwrap().address)
    }
}

impl std::cmp::PartialOrd for Validator {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Generator<validator::Info> for Validator {
    fn merge_with_default(self, default: Self) -> Self {
        Validator {
            id: self.id.or(default.id),
            voting_power: self.voting_power.or(default.voting_power),
            proposer_priority: self.proposer_priority.or(default.proposer_priority),
        }
    }

    fn generate(&self) -> Result<validator::Info, SimpleError> {
        let verification_key = self.get_private_key()?.verification_key();
        let info = validator::Info {
            address: account::Id::from(verification_key),
            pub_key: PublicKey::from(verification_key),
            power: vote::Power::try_from(self.voting_power.unwrap_or(0)).unwrap(),
            name: None,
            proposer_priority: validator::ProposerPriority::from(
                self.proposer_priority.unwrap_or_default(),
            ),
        };
        Ok(info)
    }
}

/// A helper function to generate multiple validators at once.
pub fn generate_validators(vals: &[Validator]) -> Result<Vec<validator::Info>, SimpleError> {
    sort_validators(vals)
        .iter()
        .map(|v| v.generate())
        .collect::<Result<Vec<validator::Info>, SimpleError>>()
}

/// A helper function to sort validators according to the Tendermint specs.
pub fn sort_validators(vals: &[Validator]) -> Vec<Validator> {
    let mut sorted = vals.to_owned();
    sorted.sort_by_key(|v| {
        let v = v.generate().unwrap();
        (std::cmp::Reverse(v.power), v.address)
    });
    sorted
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_publickey(pk_string: &str) -> PublicKey {
        serde_json::from_str(pk_string).unwrap()
    }

    // make a validator from a pubkey, a voting power, and a proposer priority
    fn make_validator(pk: PublicKey, vp: u64, pp: Option<i64>) -> validator::Info {
        let mut info = validator::Info::new(pk, vote::Power::try_from(vp).unwrap());
        info.proposer_priority = validator::ProposerPriority::from(pp.unwrap_or_default());
        info
    }

    #[test]
    fn test_validator() {
        let pk_a = make_publickey("{\"type\":\"tendermint/PubKeyEd25519\",\"value\":\"YnT69eNDaRaNU7teDTcyBedSD0B/Ziqx+sejm0wQba0=\"}");
        let pk_b = make_publickey("{\"type\":\"tendermint/PubKeyEd25519\",\"value\":\"hYkrBnbzZQd3r/bjZgyxXfcxfNrYg8PCVsB4JLUB9eU=\"}");

        let val = Validator::new("a").voting_power(10);
        assert_eq!(val.generate().unwrap(), make_validator(pk_a, 10, None));

        let val = val.voting_power(20);
        assert_eq!(val.generate().unwrap(), make_validator(pk_a, 20, None));

        let val = val.proposer_priority(100);
        assert_eq!(val.generate().unwrap(), make_validator(pk_a, 20, Some(100)));

        let val_b = val.id("b").proposer_priority(-100);
        assert_eq!(
            val_b.generate().unwrap(),
            make_validator(pk_b, 20, Some(-100))
        );

        let val_a = Validator::new("a").voting_power(20).proposer_priority(-100);
        assert_eq!(
            val_a.generate().unwrap(),
            make_validator(pk_a, 20, Some(-100))
        );

        let val_b_a = val_b.id("a");
        assert_eq!(val_b_a, val_a);
        assert_eq!(val_b_a.generate().unwrap(), val_a.generate().unwrap());

        let mut val = val_a;
        val.proposer_priority = None;
        assert_eq!(val.generate().unwrap(), make_validator(pk_a, 20, None));

        let mut block_val = val.generate().unwrap();

        block_val.power = vote::Power::from(30_u32);
        assert_ne!(val.generate().unwrap(), block_val);

        let val = val.voting_power(30);
        assert_eq!(val.generate().unwrap(), block_val);

        block_val.proposer_priority = validator::ProposerPriority::from(1000);
        assert_ne!(val.generate().unwrap(), block_val);

        let val = val.proposer_priority(1000);
        assert_eq!(val.generate().unwrap(), block_val);
    }
}
