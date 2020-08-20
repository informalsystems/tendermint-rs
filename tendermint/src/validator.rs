//! Tendermint validators

use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use subtle_encoding::base64;

use crate::amino_types::message::AminoMessage;
use crate::{account, hash::Hash, merkle, vote, Error, PublicKey, Signature};

/// Validator set contains a vector of validators
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Set {
    #[serde(deserialize_with = "parse_vals")]
    validators: Vec<Info>,
}

impl Set {
    /// Create a new validator set.
    /// vals is mutable so it can be sorted by address.
    pub fn new(mut vals: Vec<Info>) -> Set {
        Self::sort_validators(&mut vals);
        Set { validators: vals }
    }

    /// Get Info of the underlying validators.
    pub fn validators(&self) -> &Vec<Info> {
        &self.validators
    }

    /// Sort the validators according to the current Tendermint requirements
    /// (v. 0.33 -> by validator address, ascending)
    fn sort_validators(vals: &mut Vec<Info>) {
        vals.sort_by_key(|v| v.address);
    }

    /// Returns the validator with the given Id if its in the Set.
    pub fn validator(&self, val_id: account::Id) -> Option<Info> {
        self.validators
            .iter()
            .find(|val| val.address == val_id)
            .cloned()
    }

    /// Compute the hash of this validator set
    pub fn hash(&self) -> Hash {
        let validator_bytes: Vec<Vec<u8>> = self
            .validators()
            .iter()
            .map(|validator| validator.hash_bytes())
            .collect();

        Hash::Sha256(merkle::simple_hash_from_byte_vectors(validator_bytes))
    }

    /// Compute the total voting power within this validator set
    pub fn total_power(&self) -> u64 {
        self.validators().iter().fold(0u64, |total, val_info| {
            total + val_info.voting_power.value()
        })
    }
}

// TODO: maybe add a type (with an Option<Vec<Info>> field) instead
// for light client integration tests only
fn parse_vals<'de, D>(d: D) -> Result<Vec<Info>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut vals: Vec<Info> =
        Deserialize::deserialize(d).map(|x: Option<_>| x.unwrap_or_default())?;
    Set::sort_validators(&mut vals);
    Ok(vals)
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
    pub fn verify_signature(&self, sign_bytes: &[u8], signature: &Signature) -> Result<(), Error> {
        self.pub_key.verify(sign_bytes, signature)
    }
}

impl From<PublicKey> for account::Id {
    fn from(pub_key: PublicKey) -> account::Id {
        match pub_key {
            PublicKey::Ed25519(pk) => account::Id::from(pk),
            #[cfg(feature = "secp256k1")]
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

// Copied from tendermint_proto::types::SimpleValidator
/// SimpleValidator is the form of the validator used for computing the Merkle tree.
/// It does not include the address, as that is redundant with the pubkey,
/// nor the proposer priority, as that changes with every block even if the validator set didn't.
/// It contains only the pubkey and the voting power, and is amino encoded.
/// TODO: currently only works for Ed25519 pubkeys
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SimpleValidator {
    ///
    #[prost(message, optional, tag = "1")]
    pub pub_key: ::std::option::Option<tendermint_proto::crypto::PublicKey>,
    ///
    #[prost(int64, tag = "2")]
    pub voting_power: i64,
}

/// Info -> SimpleValidator
impl From<&Info> for SimpleValidator {
    fn from(info: &Info) -> SimpleValidator {
        SimpleValidator {
            pub_key: Some(tendermint_proto::crypto::PublicKey {
                sum: Some(tendermint_proto::crypto::public_key::Sum::Ed25519(
                    info.pub_key.to_bytes(),
                )),
            }),
            voting_power: info.voting_power.value() as i64,
        }
    }
}

impl Info {
    /// Returns the bytes to be hashed into the Merkle tree -
    /// the leaves of the tree. this is an amino encoding of the
    /// pubkey and voting power, so it includes the pubkey's amino prefix.
    pub fn hash_bytes(&self) -> Vec<u8> {
        AminoMessage::bytes_vec(&SimpleValidator::from(self))
    }
}

/// Proposer priority
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct ProposerPriority(i64);

impl ProposerPriority {
    /// Create a new Priority
    pub fn new(p: i64) -> ProposerPriority {
        ProposerPriority(p)
    }

    /// Get the current proposer priority
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
    #[serde(default)]
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

    use super::*;

    // make a validator from a hex ed25519 pubkey and a voting power
    fn make_validator(pk_string: &str, vp: u64) -> Info {
        let bytes = hex::decode_upper(pk_string).unwrap();
        let pk = PublicKey::from_raw_ed25519(&bytes).unwrap();
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
            "76A2B3F5CBB567F0D689D9DF7155FC89A4C878F040D7A5BB85FF68B74D253FC7",
            770_561_664_770_006_272,
        );
        let hash_string = "7B7B00C03EBA5ED17923243D5F7CF974BE2499522EF5B92A3EC60E868A0CCA19";
        let hash_expect = hex::decode_upper(hash_string).unwrap();

        let val_set = Set::new(vec![v1, v2, v3]);
        let hash = val_set.hash();
        assert_eq!(hash_expect, hash.as_bytes().to_vec());

        let not_in_set = make_validator(
            "76A2B3F5CBB567F0D689D9DF7155FC89A4C878F040D7A5BB85FF68B74D253FC9",
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
