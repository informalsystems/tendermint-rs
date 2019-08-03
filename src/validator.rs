//! Tendermint validators

use crate::{account, merkle, vote, PublicKey};
use prost::Message;
#[cfg(feature = "serde")]
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
#[cfg(feature = "rpc")]
use subtle_encoding::base64;

/// Validator set contains a vector of validators
#[derive(Debug)]
pub struct Set {
    validators: Vec<Info>,
}

impl Set {
    /// Create a new validator set.
    /// vals is mutable so it can be sorted by address.
    pub fn new(mut vals: Vec<Info>) -> Set {
        vals.sort_by(|v1, v2| v1.address.partial_cmp(&v2.address).unwrap());
        Set { validators: vals }
    }

    /// Compute the Merkle root of the validator set
    pub fn hash(self) -> merkle::Hash {
        // We need to get from Vec<Info> to &[&[u8]] so we can call simple_hash_from_byte_slices.
        // This looks like: Vec<Info> -> Vec<Vec<u8>> -> Vec<&[u8]> -> &[&[u8]]
        // Can we simplify this?
        // Perhaps simple_hash_from_byteslices should take Vec<Vec<u8>> directly ?
        let validator_bytes: Vec<Vec<u8>> = self
            .validators
            .into_iter()
            .map(|x| x.hash_bytes())
            .collect();
        let validator_byteslices: Vec<&[u8]> = (&validator_bytes)
            .into_iter()
            .map(|x| x.as_slice())
            .collect();
        merkle::simple_hash_from_byte_slices(validator_byteslices.as_slice())
    }
}

/// Validator information
#[derive(Clone, Debug)]
pub struct Info {
    /// Validator account address
    pub address: account::Id,

    /// Validator public key
    pub pub_key: PublicKey,

    /// Validator voting power
    pub voting_power: vote::Power,

    /// Validator proposer priority
    pub proposer_priority: Option<ProposerPriority>,
}

/// InfoHashable is the form of the validator used for computing the Merkle tree.
/// It does not include the address, as that is redundant with the pubkey,
/// nor the proposer priority, as that changes with every block even if the validator set didn't.
#[derive(Clone, PartialEq, Message)]
struct InfoHashable {
    #[prost(bytes, tag = "1", amino_name = "tendermint/PubKeyEd25519")]
    pub pub_key: Vec<u8>,
    #[prost(uint64, tag = "2")]
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

// returns the bytes to be hashed into the Merkle tree -
// the leaves of the tree.
impl Info {
    fn hash_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        InfoHashable::from(self).encode(&mut bytes).unwrap();
        bytes
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

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for ProposerPriority {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(ProposerPriority(
            String::deserialize(deserializer)?
                .parse()
                .map_err(|e| D::Error::custom(format!("{}", e)))?,
        ))
    }
}

#[cfg(feature = "serde")]
impl Serialize for ProposerPriority {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.to_string().serialize(serializer)
    }
}

/// Updates to the validator set
#[cfg(feature = "rpc")]
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
#[cfg(feature = "rpc")]
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum PK {
    /// Ed25519 keys
    #[serde(rename = "ed25519")]
    Ed25519(String),
}

#[cfg(feature = "rpc")]
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
    use crate::account::Id;
    use signatory::{ed25519, PublicKeyed};
    use signatory_dalek;

    fn generate_validator(vp: u64) -> Info {
        let seed = ed25519::Seed::generate();
        let signer = signatory_dalek::Ed25519Signer::from(&seed);
        let pk = signer.public_key().unwrap();
        Info {
            address: Id::from(pk),
            pub_key: PublicKey::Ed25519(pk),
            voting_power: vote::Power::new(vp),
            proposer_priority: None,
        }
    }

    #[test]
    fn test_validator_set() {
        // TODO: get a test vector from the Go code instead of generating validators
        let v1 = generate_validator(1);
        let v2 = generate_validator(2);
        let v3 = generate_validator(3);
        println!("{:?}", v1.address);
        println!("{:?}", v2.address);
        println!("{:?}", v3.address);

        let val_set = Set::new(vec![v1, v2, v3]);
        println!("{:?}", val_set);
        let hash = val_set.hash();
        println!("{:?}", hash);
    }
}
