//! Public keys used in Tendermint networks

use crate::error::{Error, ErrorKind};
#[cfg(feature = "serde")]
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use signatory::{ecdsa::curve::secp256k1, ed25519};
use std::{
    fmt::{self, Display},
    ops::Deref,
    str::FromStr,
};
#[cfg(feature = "serde")]
use subtle_encoding::base64;
use subtle_encoding::{bech32, hex};

/// Public keys allowed in Tendermint protocols
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum PublicKey {
    /// Ed25519 keys
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "tendermint/PubKeyEd25519",
            serialize_with = "serialize_ed25519_base64",
            deserialize_with = "deserialize_ed25519_base64"
        )
    )]
    Ed25519(ed25519::PublicKey),

    /// Secp256k1 keys
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "tendermint/PubKeySecp256k1",
            serialize_with = "serialize_secp256k1_base64",
            deserialize_with = "deserialize_secp256k1_base64"
        )
    )]
    Secp256k1(secp256k1::PublicKey),
}

impl PublicKey {
    /// From raw secp256k1 public key bytes
    pub fn from_raw_secp256k1(bytes: &[u8]) -> Option<PublicKey> {
        Some(PublicKey::Secp256k1(secp256k1::PublicKey::from_bytes(
            bytes,
        )?))
    }

    /// From raw Ed25519 public key bytes
    pub fn from_raw_ed25519(bytes: &[u8]) -> Option<PublicKey> {
        Some(PublicKey::Ed25519(ed25519::PublicKey::from_bytes(bytes)?))
    }

    /// Get Ed25519 public key
    pub fn ed25519(self) -> Option<ed25519::PublicKey> {
        match self {
            PublicKey::Ed25519(pk) => Some(pk),
            _ => None,
        }
    }

    /// Serialize this key as amino bytes
    pub fn to_amino_bytes(self) -> Vec<u8> {
        match self {
            PublicKey::Ed25519(ref pk) => {
                //Amino prefix for Pubkey
                let mut key_bytes = vec![0x16, 0x24, 0xDE, 0x64, 0x20];
                key_bytes.extend(pk.as_bytes());
                key_bytes
            }
            PublicKey::Secp256k1(ref pk) => {
                let mut key_bytes = vec![0xEB, 0x5A, 0xE9, 0x87, 0x21];
                key_bytes.extend(pk.as_bytes());
                key_bytes
            }
        }
    }

    /// Serialize this key as Bech32 with the given human readable prefix
    pub fn to_bech32(self, hrp: &str) -> String {
        bech32::encode(hrp, self.to_amino_bytes())
    }

    /// Serialize this key as hexadecimal
    pub fn to_hex(self) -> String {
        String::from_utf8(hex::encode_upper(self.to_amino_bytes())).unwrap()
    }
}

impl From<ed25519::PublicKey> for PublicKey {
    fn from(pk: ed25519::PublicKey) -> PublicKey {
        PublicKey::Ed25519(pk)
    }
}

impl From<secp256k1::PublicKey> for PublicKey {
    fn from(pk: secp256k1::PublicKey) -> PublicKey {
        PublicKey::Secp256k1(pk)
    }
}

/// Public key roles used in Tendermint networks
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum TendermintKey {
    /// User signing keys used for interacting with accounts in the state machine
    AccountKey(PublicKey),

    /// Validator signing keys used for authenticating consensus protocol messages
    ConsensusKey(PublicKey),
}

impl TendermintKey {
    /// Create a new account key from a `PublicKey`
    pub fn new_account_key(public_key: PublicKey) -> Result<TendermintKey, Error> {
        match public_key {
            PublicKey::Ed25519(_) | PublicKey::Secp256k1(_) => {
                Ok(TendermintKey::AccountKey(public_key))
            }
        }
    }

    /// Create a new consensus key from a `PublicKey`
    pub fn new_consensus_key(public_key: PublicKey) -> Result<TendermintKey, Error> {
        match public_key {
            PublicKey::Ed25519(_) => Ok(TendermintKey::AccountKey(public_key)),
            _ => Err(err!(
                ErrorKind::InvalidKey,
                "only ed25519 consensus keys are supported"
            )),
        }
    }
}

impl Deref for TendermintKey {
    type Target = PublicKey;

    fn deref(&self) -> &PublicKey {
        match self {
            TendermintKey::AccountKey(key) => key,
            TendermintKey::ConsensusKey(key) => key,
        }
    }
}

/// Public key algorithms
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Algorithm {
    /// ed25519
    Ed25519,

    /// secp256k1
    Secp256k1,
}

impl Algorithm {
    /// Get the string label for this algorithm
    pub fn as_str(&self) -> &str {
        match self {
            Algorithm::Ed25519 => "ed25519",
            Algorithm::Secp256k1 => "secp256k1",
        }
    }
}

impl Display for Algorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Algorithm {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "ed25519" => Ok(Algorithm::Ed25519),
            "secp256k1" => Ok(Algorithm::Secp256k1),
            _ => Err(ErrorKind::Parse.into()),
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for Algorithm {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_str().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Algorithm {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::from_str(&String::deserialize(deserializer)?)
            .map_err(|e| D::Error::custom(format!("{}", e)))
    }
}

/// Serialize the bytes of an Ed25519 public key as Base64. Used for serializing JSON
#[cfg(feature = "serde")]
fn serialize_ed25519_base64<S>(pk: &ed25519::PublicKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    String::from_utf8(base64::encode(pk.as_bytes()))
        .unwrap()
        .serialize(serializer)
}

/// Serialize the bytes of a secp256k1 ECDSA public key as Base64. Used for serializing JSON
#[cfg(feature = "serde")]
fn serialize_secp256k1_base64<S>(
    pk: &secp256k1::PublicKey,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    String::from_utf8(base64::encode(pk.as_bytes()))
        .unwrap()
        .serialize(serializer)
}

#[cfg(feature = "serde")]
fn deserialize_ed25519_base64<'de, D>(deserializer: D) -> Result<ed25519::PublicKey, D::Error>
where
    D: Deserializer<'de>,
{
    let bytes = base64::decode(String::deserialize(deserializer)?.as_bytes())
        .map_err(|e| D::Error::custom(format!("{}", e)))?;

    ed25519::PublicKey::from_bytes(&bytes).ok_or_else(|| D::Error::custom("invalid ed25519 key"))
}

#[cfg(feature = "serde")]
fn deserialize_secp256k1_base64<'de, D>(
    deserializer: D,
) -> Result<signatory::ecdsa::curve::secp256k1::PublicKey, D::Error>
where
    D: Deserializer<'de>,
{
    let bytes = base64::decode(String::deserialize(deserializer)?.as_bytes())
        .map_err(|e| D::Error::custom(format!("{}", e)))?;

    secp256k1::PublicKey::from_bytes(&bytes)
        .ok_or_else(|| D::Error::custom("invalid secp256k1 key"))
}

#[cfg(test)]
mod tests {
    use super::{PublicKey, TendermintKey};
    use subtle_encoding::hex;

    const EXAMPLE_CONSENSUS_KEY: &str =
        "4A25C6640A1F72B9C975338294EF51B6D1C33158BB6ECBA69FBC3FB5A33C9DCE";

    #[test]
    fn test_consensus_serialization() {
        let example_key = TendermintKey::ConsensusKey(
            PublicKey::from_raw_ed25519(&hex::decode_upper(EXAMPLE_CONSENSUS_KEY).unwrap())
                .unwrap(),
        );

        assert_eq!(
            example_key.to_bech32("cosmosvalconspub"),
            "cosmosvalconspub1zcjduepqfgjuveq2raetnjt4xwpffm63kmguxv2chdhvhf5lhslmtgeunh8qmf7exk"
        );
    }

    const EXAMPLE_ACCOUNT_KEY: &str =
        "02A1633CAFCC01EBFB6D78E39F687A1F0995C62FC95F51EAD10A02EE0BE551B5DC";

    #[test]
    fn test_account_serialization() {
        let example_key = TendermintKey::AccountKey(
            PublicKey::from_raw_secp256k1(&hex::decode_upper(EXAMPLE_ACCOUNT_KEY).unwrap())
                .unwrap(),
        );

        assert_eq!(
            example_key.to_bech32("cosmospub"),
            "cosmospub1addwnpepq2skx090esq7h7md0r3e76r6ruyet330e904r6k3pgpwuzl92x6actrt4uq"
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn json_parsing() {
        let json_string = "{\"type\":\"tendermint/PubKeyEd25519\",\"value\":\"RblzMO4is5L1hZz6wo4kPbptzOyue6LTk4+lPhD1FRk=\"}";
        let pubkey: PublicKey = serde_json::from_str(json_string).unwrap();

        assert_eq!(
            pubkey.ed25519().unwrap().as_ref(),
            [
                69, 185, 115, 48, 238, 34, 179, 146, 245, 133, 156, 250, 194, 142, 36, 61, 186,
                109, 204, 236, 174, 123, 162, 211, 147, 143, 165, 62, 16, 245, 21, 25
            ]
        );

        let reserialized_json = serde_json::to_string(&pubkey).unwrap();
        assert_eq!(reserialized_json.as_str(), json_string);
    }
}
