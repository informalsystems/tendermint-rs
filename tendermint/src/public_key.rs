//! Public keys used in Tendermint networks

pub use ed25519_dalek::PublicKey as Ed25519;

#[cfg(feature = "secp256k1")]
pub use k256::EncodedPoint as Secp256k1;

use crate::{
    error::{self, Error},
    signature::Signature,
};
use anomaly::{fail, format_err};
use serde::{de, ser, Deserialize, Serialize};
use signature::Verifier as _;
use std::{cmp::Ordering, fmt, ops::Deref, str::FromStr};
use subtle_encoding::{base64, bech32, hex};

/// Public keys allowed in Tendermint protocols
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(tag = "type", content = "value")]
pub enum PublicKey {
    /// Ed25519 keys
    #[serde(
        rename = "tendermint/PubKeyEd25519",
        serialize_with = "serialize_ed25519_base64",
        deserialize_with = "deserialize_ed25519_base64"
    )]
    Ed25519(Ed25519),

    /// Secp256k1 keys
    #[cfg(feature = "secp256k1")]
    #[cfg_attr(docsrs, doc(cfg(feature = "secp256k1")))]
    #[serde(
        rename = "tendermint/PubKeySecp256k1",
        serialize_with = "serialize_secp256k1_base64",
        deserialize_with = "deserialize_secp256k1_base64"
    )]
    Secp256k1(Secp256k1),
}

impl PublicKey {
    /// From raw secp256k1 public key bytes
    #[cfg(feature = "secp256k1")]
    #[cfg_attr(docsrs, doc(cfg(feature = "secp256k1")))]
    pub fn from_raw_secp256k1(bytes: &[u8]) -> Option<PublicKey> {
        Secp256k1::from_bytes(bytes).ok().map(PublicKey::Secp256k1)
    }

    /// From raw Ed25519 public key bytes
    pub fn from_raw_ed25519(bytes: &[u8]) -> Option<PublicKey> {
        Ed25519::from_bytes(bytes).map(Into::into).ok()
    }

    /// Get Ed25519 public key
    pub fn ed25519(self) -> Option<Ed25519> {
        #[allow(unreachable_patterns)]
        match self {
            PublicKey::Ed25519(pk) => Some(pk),
            _ => None,
        }
    }

    /// Get Secp256k1 public key
    #[cfg(feature = "secp256k1")]
    #[cfg_attr(docsrs, doc(cfg(feature = "secp256k1")))]
    pub fn secp256k1(self) -> Option<Secp256k1> {
        match self {
            PublicKey::Secp256k1(pk) => Some(pk),
            _ => None,
        }
    }

    /// Verify the given [`Signature`] using this public key
    pub fn verify(&self, msg: &[u8], signature: &Signature) -> Result<(), Error> {
        match self {
            PublicKey::Ed25519(pk) => match signature {
                Signature::Ed25519(sig) => pk.verify(msg, sig).map_err(|_| {
                    format_err!(
                        error::Kind::SignatureInvalid,
                        "Ed25519 signature verification failed"
                    )
                    .into()
                }),
            },
            #[cfg(feature = "secp256k1")]
            PublicKey::Secp256k1(_) => fail!(
                error::Kind::InvalidKey,
                "unsupported signature algorithm (ECDSA/secp256k1)"
            ),
        }
    }

    /// View this key as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            PublicKey::Ed25519(pk) => pk.as_bytes(),
            #[cfg(feature = "secp256k1")]
            PublicKey::Secp256k1(pk) => pk.as_bytes(),
        }
    }

    /// Get a vector containing the byte serialization of this key
    pub fn to_bytes(self) -> Vec<u8> {
        self.as_bytes().to_vec()
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
            #[cfg(feature = "secp256k1")]
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

impl From<Ed25519> for PublicKey {
    fn from(pk: Ed25519) -> PublicKey {
        PublicKey::Ed25519(pk)
    }
}

#[cfg(feature = "secp256k1")]
impl From<Secp256k1> for PublicKey {
    fn from(pk: Secp256k1) -> PublicKey {
        PublicKey::Secp256k1(pk)
    }
}

impl PartialOrd for PublicKey {
    fn partial_cmp(&self, other: &PublicKey) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PublicKey {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            PublicKey::Ed25519(a) => match other {
                PublicKey::Ed25519(b) => a.as_bytes().cmp(b.as_bytes()),
                #[cfg(feature = "secp256k1")]
                PublicKey::Secp256k1(_) => Ordering::Less,
            },
            #[cfg(feature = "secp256k1")]
            PublicKey::Secp256k1(a) => match other {
                PublicKey::Ed25519(_) => Ordering::Greater,
                #[cfg(feature = "secp256k1")]
                PublicKey::Secp256k1(b) => a.as_bytes().cmp(b.as_bytes()),
            },
        }
    }
}

/// Public key roles used in Tendermint networks
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum TendermintKey {
    /// User signing keys used for interacting with accounts in the state machine
    AccountKey(PublicKey),

    /// Validator signing keys used for authenticating consensus protocol messages
    ConsensusKey(PublicKey),
}

impl TendermintKey {
    /// Create a new account key from a [`PublicKey`]
    pub fn new_account_key(public_key: PublicKey) -> Result<TendermintKey, Error> {
        match public_key {
            PublicKey::Ed25519(_) => Ok(TendermintKey::AccountKey(public_key)),
            #[cfg(feature = "secp256k1")]
            PublicKey::Secp256k1(_) => Ok(TendermintKey::AccountKey(public_key)),
        }
    }

    /// Create a new consensus key from a [`PublicKey`]
    pub fn new_consensus_key(public_key: PublicKey) -> Result<TendermintKey, Error> {
        #[allow(unreachable_patterns)]
        match public_key {
            PublicKey::Ed25519(_) => Ok(TendermintKey::AccountKey(public_key)),
            _ => fail!(
                error::Kind::InvalidKey,
                "only ed25519 consensus keys are supported"
            ),
        }
    }

    /// Get the [`PublicKey`] value for this [`TendermintKey`]
    pub fn public_key(&self) -> &PublicKey {
        match self {
            TendermintKey::AccountKey(key) => key,
            TendermintKey::ConsensusKey(key) => key,
        }
    }
}

// TODO(tarcieri): deprecate/remove this in favor of `TendermintKey::public_key`
impl Deref for TendermintKey {
    type Target = PublicKey;

    fn deref(&self) -> &PublicKey {
        self.public_key()
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

impl fmt::Display for Algorithm {
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
            _ => Err(error::Kind::Parse.into()),
        }
    }
}

impl Serialize for Algorithm {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_str().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Algorithm {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use de::Error;
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(D::Error::custom)
    }
}

/// Serialize the bytes of an Ed25519 public key as Base64. Used for serializing JSON
fn serialize_ed25519_base64<S>(pk: &Ed25519, serializer: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    String::from_utf8(base64::encode(pk.as_bytes()))
        .unwrap()
        .serialize(serializer)
}

/// Serialize the bytes of a secp256k1 ECDSA public key as Base64. Used for serializing JSON
#[cfg(feature = "secp256k1")]
fn serialize_secp256k1_base64<S>(pk: &Secp256k1, serializer: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    String::from_utf8(base64::encode(pk.as_bytes()))
        .unwrap()
        .serialize(serializer)
}

fn deserialize_ed25519_base64<'de, D>(deserializer: D) -> Result<Ed25519, D::Error>
where
    D: de::Deserializer<'de>,
{
    use de::Error;
    let encoded = String::deserialize(deserializer)?;
    let bytes = base64::decode(&encoded).map_err(D::Error::custom)?;
    Ed25519::from_bytes(&bytes).map_err(D::Error::custom)
}

#[cfg(feature = "secp256k1")]
fn deserialize_secp256k1_base64<'de, D>(deserializer: D) -> Result<Secp256k1, D::Error>
where
    D: de::Deserializer<'de>,
{
    use de::Error;
    let encoded = String::deserialize(deserializer)?;
    let bytes = base64::decode(&encoded).map_err(D::Error::custom)?;
    Secp256k1::from_bytes(&bytes).map_err(|_| D::Error::custom("invalid secp256k1 key"))
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

    #[test]
    #[cfg(feature = "secp256k1")]
    fn test_account_serialization() {
        const EXAMPLE_ACCOUNT_KEY: &str =
            "02A1633CAFCC01EBFB6D78E39F687A1F0995C62FC95F51EAD10A02EE0BE551B5DC";
        let example_key = TendermintKey::AccountKey(
            PublicKey::from_raw_secp256k1(&hex::decode_upper(EXAMPLE_ACCOUNT_KEY).unwrap())
                .unwrap(),
        );

        assert_eq!(
            example_key.to_bech32("cosmospub"),
            "cosmospub1addwnpepq2skx090esq7h7md0r3e76r6ruyet330e904r6k3pgpwuzl92x6actrt4uq"
        );
    }

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
