//! Tendermint validators

use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use subtle_encoding::base64;

use crate::amino_types::message::AminoMessage;
use crate::{account, hash::Hash, merkle, vote, Error, PublicKey, Signature};

use std::convert::TryFrom;
use tendermint_proto::types::SimpleValidator as RawSimpleValidator;
use tendermint_proto::DomainType;

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
    /// (v. 0.34 -> by validator power, descending)
    fn sort_validators(vals: &mut Vec<Info>) {
        vals.sort_by_key(|v| std::cmp::Reverse(v.voting_power));
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

/// SimpleValidator is the form of the validator used for computing the Merkle tree.
/// It does not include the address, as that is redundant with the pubkey,
/// nor the proposer priority, as that changes with every block even if the validator set didn't.
/// It contains only the pubkey and the voting power, and is amino encoded.
/// TODO: currently only works for Ed25519 pubkeys
#[derive(Clone, PartialEq, DomainType)]
#[rawtype(RawSimpleValidator)]
pub struct SimpleValidator {
    /// Public key
    pub pub_key: Option<tendermint_proto::crypto::PublicKey>,
    /// Voting power
    pub voting_power: i64,
}

impl TryFrom<RawSimpleValidator> for SimpleValidator {
    type Error = Error;

    fn try_from(value: RawSimpleValidator) -> Result<Self, Self::Error> {
        Ok(SimpleValidator {
            pub_key: value.pub_key,
            voting_power: value.voting_power,
        })
    }
}

impl From<SimpleValidator> for RawSimpleValidator {
    fn from(value: SimpleValidator) -> Self {
        RawSimpleValidator {
            pub_key: value.pub_key,
            voting_power: value.voting_power,
        }
    }
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
        let raw_simple_validator: RawSimpleValidator = SimpleValidator::from(self).into();
        AminoMessage::bytes_vec(&raw_simple_validator)
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

    use super::*;

    // make a validator
    fn make_validator(pk: Vec<u8>, vp: u64) -> Info {
        let pk = PublicKey::from_raw_ed25519(&pk).unwrap();
        Info::new(pk, vote::Power::new(vp))
    }

    #[test]
    fn test_validator_set() {
        // test vector generated by Go code
        /*
           import (
               "fmt"
               "github.com/tendermint/tendermint/crypto/ed25519"
               "github.com/tendermint/tendermint/types"
               "strings"
           )
           func testValSet() {
               pk1 := ed25519.GenPrivKeyFromSecret([]byte{4, 211, 14, 157, 10, 0, 205, 9, 10, 116, 207, 161, 4, 211, 190, 37, 108, 88, 202, 168, 63, 135, 0, 141, 53, 55, 254, 57, 40, 184, 20, 242})
               pk2 := ed25519.GenPrivKeyFromSecret([]byte{99, 231, 126, 151, 159, 236, 2, 229, 33, 44, 200, 248, 147, 176, 13, 127, 105, 76, 49, 83, 25, 101, 44, 57, 20, 215, 166, 188, 134, 94, 56, 165})
               pk3 := ed25519.GenPrivKeyFromSecret([]byte{54, 253, 151, 16, 182, 114, 125, 12, 74, 101, 54, 253, 174, 153, 121, 74, 145, 180, 111, 16, 214, 48, 193, 109, 104, 134, 55, 162, 151, 16, 182, 114})
               not_in_set := ed25519.GenPrivKeyFromSecret([]byte{121, 74, 145, 180, 111, 16, 214, 48, 193, 109, 35, 68, 19, 27, 173, 69, 92, 204, 127, 218, 234, 81, 232, 75, 204, 199, 48, 163, 55, 132, 231, 147})
               fmt.Println("pk1: ", strings.Join(strings.Split(fmt.Sprintf("%v", pk1.PubKey().Bytes()), " "), ", "))
               fmt.Println("pk2:", strings.Join(strings.Split(fmt.Sprintf("%v", pk2.PubKey().Bytes()), " "), ", "))
               fmt.Println("pk3: ", strings.Join(strings.Split(fmt.Sprintf("%v", pk3.PubKey().Bytes()), " "), ", "))
               fmt.Println("not_in_set: ", strings.Join(strings.Split(fmt.Sprintf("%v", not_in_set.PubKey().Bytes()), " "), ", "))
               v1 := types.NewValidator(pk1.PubKey(), 148151478422287875)
               v2 := types.NewValidator(pk2.PubKey(), 158095448483785107)
               v3 := types.NewValidator(pk3.PubKey(), 770561664770006272)
               set := types.NewValidatorSet([]*types.Validator{v1, v2, v3})
               fmt.Println("Hash:", strings.Join(strings.Split(fmt.Sprintf("%v", set.Hash()), " "), ", "))
           }
        */
        let v1 = make_validator(
            vec![
                48, 163, 55, 132, 231, 147, 230, 163, 56, 158, 127, 218, 179, 139, 212, 103, 218,
                89, 122, 126, 229, 88, 84, 48, 32, 0, 185, 174, 63, 72, 203, 52,
            ],
            148_151_478_422_287_875,
        );
        let v2 = make_validator(
            vec![
                54, 253, 174, 153, 121, 74, 145, 180, 111, 16, 214, 48, 193, 109, 104, 134, 55,
                162, 151, 16, 182, 114, 125, 135, 32, 195, 236, 248, 64, 112, 74, 101,
            ],
            158_095_448_483_785_107,
        );
        let v3 = make_validator(
            vec![
                182, 205, 13, 86, 147, 27, 65, 49, 160, 118, 11, 180, 117, 35, 206, 35, 68, 19, 27,
                173, 69, 92, 204, 224, 200, 51, 249, 81, 105, 128, 112, 244,
            ],
            770_561_664_770_006_272,
        );
        let hash_expect = vec![
            11, 64, 107, 4, 234, 81, 232, 75, 204, 199, 160, 114, 229, 97, 243, 95, 118, 213, 17,
            22, 57, 84, 71, 122, 200, 169, 192, 252, 41, 148, 223, 180,
        ];

        let val_set = Set::new(vec![v1, v2, v3]);
        let hash = val_set.hash();
        assert_eq!(hash_expect, hash.as_bytes().to_vec());

        let not_in_set = make_validator(
            vec![
                110, 147, 87, 120, 27, 218, 66, 209, 81, 4, 169, 153, 64, 163, 137, 89, 168, 97,
                219, 233, 42, 119, 24, 61, 47, 59, 76, 31, 182, 60, 13, 4,
            ],
            10_000_000_000_000_000,
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
