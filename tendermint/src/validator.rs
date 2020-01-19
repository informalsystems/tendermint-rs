//! Tendermint validators

use prost_amino_derive::Message;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use signatory::{
    ed25519,
    signature::{Signature, Verifier},
};
use signatory_dalek::Ed25519Verifier;
use subtle_encoding::base64;

use crate::amino_types::message::AminoMessage;
use crate::{account, vote, PublicKey};

/// Validator set contains a vector of validators
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Set {
    #[serde(deserialize_with = "parse_vals")]
    validators: Vec<Info>,
}

impl Set {
    /// Create a new validator set.
    /// vals is mutable so it can be sorted by address.
    pub fn new(mut vals: Vec<Info>) -> Set {
        vals.sort_by(|v1, v2| v1.address.partial_cmp(&v2.address).unwrap());
        Set { validators: vals }
    }

    /// Get Info of the underlying validators.
    pub fn validators(&self) -> &Vec<Info> {
        &self.validators
    }
}

impl Set {
    /// Returns the validator with the given Id if its in the Set.
    pub fn validator(&self, val_id: account::Id) -> Option<Info> {
        self.validators
            .iter()
            .find(|val| val.address == val_id)
            .cloned()
    }
}

// TODO: maybe add a type (with an Option<Vec<Info>> field) instead
// for light client integration tests only
fn parse_vals<'de, D>(d: D) -> Result<Vec<Info>, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(d).map(|x: Option<_>| x.unwrap_or_default())
}

/// Validator information
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct Info {
    /// Validator account address
    pub address: account::Id,

    /// Validator public key
    pub pub_key: PublicKey,

    /// Validator voting power
    #[serde(alias = "power")]
    pub voting_power: vote::Power,

    /// Validator proposer priority
    pub proposer_priority: Option<ProposerPriority>,
}

impl Info {
    /// Return the voting power of the validator.
    pub fn power(&self) -> u64 {
        self.voting_power.value()
    }

    /// Verify the given signature against the given sign_bytes using the validators
    /// public key.
    pub fn verify_signature(&self, sign_bytes: &[u8], signature: &[u8]) -> bool {
        if let Some(pk) = &self.pub_key.ed25519() {
            let verifier = Ed25519Verifier::from(pk);
            if let Ok(sig) = ed25519::Signature::from_bytes(signature) {
                return verifier.verify(sign_bytes, &sig).is_ok();
            }
        }
        false
    }
}

impl From<PublicKey> for account::Id {
    fn from(pub_key: PublicKey) -> account::Id {
        match pub_key {
            PublicKey::Ed25519(pk) => account::Id::from(pk),
            PublicKey::Secp256k1(pk) => account::Id::from(pk),
        }
    }
}

impl Info {
    /// Create a new validator.
    pub fn new(pk: PublicKey, vp: vote::Power) -> Info {
        Info {
            address: account::Id::from(pk),
            pub_key: pk,
            voting_power: vp,
            proposer_priority: None,
        }
    }
}

/// InfoHashable is the form of the validator used for computing the Merkle tree.
/// It does not include the address, as that is redundant with the pubkey,
/// nor the proposer priority, as that changes with every block even if the validator set didn't.
/// It contains only the pubkey and the voting power, and is amino encoded.
/// TODO: currently only works for Ed25519 pubkeys
#[derive(Clone, PartialEq, Message)]
struct InfoHashable {
    #[prost_amino(bytes, tag = "1", amino_name = "tendermint/PubKeyEd25519")]
    pub pub_key: Vec<u8>,
    #[prost_amino(uint64, tag = "2")]
    voting_power: u64,
}

/// Info -> InfoHashable
impl From<&Info> for InfoHashable {
    fn from(info: &Info) -> InfoHashable {
        InfoHashable {
            pub_key: info.pub_key.as_bytes(),
            voting_power: info.voting_power.value(),
        }
    }
}

impl Info {
    /// Returns the bytes to be hashed into the Merkle tree -
    /// the leaves of the tree. this is an amino encoding of the
    /// pubkey and voting power, so it includes the pubkey's amino prefix.
    pub fn hash_bytes(&self) -> Vec<u8> {
        AminoMessage::bytes_vec(&InfoHashable::from(self))
    }
}

/// Proposer priority
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct ProposerPriority(i64);

impl ProposerPriority {
    /// Get the current voting power
    pub fn value(self) -> i64 {
        self.0
    }
}

impl From<ProposerPriority> for i64 {
    fn from(priority: ProposerPriority) -> i64 {
        priority.value()
    }
}

impl<'de> Deserialize<'de> for ProposerPriority {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(ProposerPriority(
            String::deserialize(deserializer)?
                .parse()
                .map_err(|e| D::Error::custom(format!("{}", e)))?,
        ))
    }
}

impl Serialize for ProposerPriority {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.to_string().serialize(serializer)
    }
}

/// Updates to the validator set
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Update {
    /// Validator public key
    #[serde(deserialize_with = "deserialize_public_key")]
    pub pub_key: PublicKey,

    /// New voting power
    pub power: vote::Power,
}

/// Validator updates use a slightly different public key format than the one
/// implemented in `tendermint::PublicKey`.
///
/// This is an internal thunk type to parse the `validator_updates` format and
/// then convert to `tendermint::PublicKey` in `deserialize_public_key` below.
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum PK {
    /// Ed25519 keys
    #[serde(rename = "ed25519")]
    Ed25519(String),
}

fn deserialize_public_key<'de, D>(deserializer: D) -> Result<PublicKey, D::Error>
where
    D: Deserializer<'de>,
{
    match &PK::deserialize(deserializer)? {
        PK::Ed25519(base64_value) => {
            let bytes =
                base64::decode(base64_value).map_err(|e| D::Error::custom(format!("{}", e)))?;

            PublicKey::from_raw_ed25519(&bytes)
                .ok_or_else(|| D::Error::custom("error parsing Ed25519 key"))
        }
    }
}

#[cfg(test)]
mod tests {
    use subtle_encoding::hex;

    use crate::lite::ValidatorSet;

    use super::*;

    // make a validator from a hex ed25519 pubkey and a voting power
    fn make_validator(pk_string: &str, vp: u64) -> Info {
        let pk = PublicKey::from_raw_ed25519(&hex::decode_upper(pk_string).unwrap()).unwrap();
        Info::new(pk, vote::Power::new(vp))
    }

    #[test]
    fn test_validator_set() {
        // test vector generated by Go code
        let v1 = make_validator(
            "F349539C7E5EF7C49549B09C4BFC2335318AB0FE51FBFAA2433B4F13E816F4A7",
            148_151_478_422_287_875,
        );
        let v2 = make_validator(
            "5646AA4C706B7AF73768903E77D117487D2584B76D83EB8FF287934EE7758AFC",
            158_095_448_483_785_107,
        );
        let v3 = make_validator(
            "EB6B732C4BD86B5FA3F3BC3DB688DA0ED182A7411F81C2D405506B298FC19E52",
            770_561_664_770_006_272,
        );
        let hash_string = "B92B4474567A1B57969375C13CF8129AA70230642BD7FB9FB2CC316E87CE01D7";
        let hash_expect = &hex::decode_upper(hash_string).unwrap();

        let val_set = Set::new(vec![v1, v2, v3]);
        let hash = val_set.hash();
        assert_eq!(hash_expect, &hash.as_bytes().to_vec());

        let not_in_set = make_validator(
            "EB6B732C5BD86B5FA3F3BC3DB688DA0ED182A7411F81C2D405506B298FC19E52",
            1,
        );
        assert_eq!(val_set.validator(v1.address).unwrap(), v1);
        assert_eq!(val_set.validator(v2.address).unwrap(), v2);
        assert_eq!(val_set.validator(v3.address).unwrap(), v3);
        assert_eq!(val_set.validator(not_in_set.address), None);
        assert_eq!(
            val_set.total_power(),
            148_151_478_422_287_875 + 158_095_448_483_785_107 + 770_561_664_770_006_272
        );
    }
}
