//! Public keys used in Tendermint networks

#[cfg(feature = "secp256k1")]
pub use k256::ecdsa::VerifyingKey as Secp256k1;

mod pub_key_request;
mod pub_key_response;

pub use pub_key_request::PubKeyRequest;
pub use pub_key_response::PubKeyResponse;

use core::convert::TryFrom;
use core::{cmp::Ordering, fmt, str::FromStr};
use serde::{de, ser, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use subtle_encoding::{base64, bech32, hex};

pub use crate::crypto::ed25519::VerificationKey as Ed25519;
use crate::{error::Error, prelude::*};

// Note:On the golang side this is generic in the sense that it could everything that implements
// github.com/tendermint/tendermint/crypto.PubKey
// While this is meant to be used with different key-types, it currently only uses a PubKeyEd25519
// version.
// TODO: make this more generic

// Warning: the custom serialization implemented here does not use TryFrom<RawPublicKey>.
//          it should only be used to read/write the priva_validator_key.json.
//          All changes to the serialization should check both the JSON and protobuf conversions.
// Todo: Merge JSON serialization with #[serde(try_from = "RawPublicKey", into = "RawPublicKey)]
/// Public keys allowed in Tendermint protocols
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(tag = "type", content = "value")] // JSON custom serialization for priv_validator_key.json
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

// Internal thunk type to facilitate deserialization from the raw Protobuf data
// structure's JSON representation.
#[derive(Serialize, Deserialize)]
struct ProtobufPublicKeyWrapper {
    #[serde(rename = "Sum")]
    sum: ProtobufPublicKey,
}

impl From<ProtobufPublicKeyWrapper> for PublicKey {
    fn from(wrapper: ProtobufPublicKeyWrapper) -> Self {
        match wrapper.sum {
            ProtobufPublicKey::Ed25519 { ed25519 } => PublicKey::Ed25519(ed25519),
            #[cfg(feature = "secp256k1")]
            ProtobufPublicKey::Secp256k1 { secp256k1 } => PublicKey::Secp256k1(secp256k1),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "value")] // JSON custom serialization for priv_validator_key.json
enum ProtobufPublicKey {
    #[serde(rename = "tendermint.crypto.PublicKey_Ed25519")]
    Ed25519 {
        #[serde(
            serialize_with = "serialize_ed25519_base64",
            deserialize_with = "deserialize_ed25519_base64"
        )]
        ed25519: Ed25519,
    },

    #[cfg(feature = "secp256k1")]
    #[serde(rename = "tendermint.crypto.PublicKey_Secp256K1")]
    Secp256k1 {
        #[serde(
            serialize_with = "serialize_secp256k1_base64",
            deserialize_with = "deserialize_secp256k1_base64"
        )]
        secp256k1: Secp256k1,
    },
}

/// Custom deserialization for public keys to handle multiple potential JSON
/// formats from Tendermint.
///
/// See <https://github.com/informalsystems/tendermint-rs/issues/1021> for
/// context.
// TODO(thane): Remove this once the serialization in Tendermint has been fixed.
pub fn deserialize_public_key<'de, D>(deserializer: D) -> Result<PublicKey, D::Error>
where
    D: Deserializer<'de>,
{
    let v = Value::deserialize(deserializer)?;
    if v.as_object()
        .map(|obj| obj.contains_key("Sum"))
        .unwrap_or(false)
    {
        serde_json::from_value::<ProtobufPublicKeyWrapper>(v).map(Into::into)
    } else {
        serde_json::from_value::<PublicKey>(v)
    }
    .map_err(serde::de::Error::custom)
}

tendermint_pb_modules! {
    use super::{PublicKey, Ed25519};
    use pb::crypto::{PublicKey as RawPublicKey, public_key::Sum};
    use crate::{prelude::*, Error};

    impl Protobuf<RawPublicKey> for PublicKey {}

    impl TryFrom<RawPublicKey> for PublicKey {
        type Error = Error;

        fn try_from(value: RawPublicKey) -> Result<Self, Self::Error> {
            let sum = &value
                .sum
                .ok_or_else(|| Error::invalid_key("empty sum".to_string()))?;
            if let Sum::Ed25519(b) = sum {
                let key = Ed25519::try_from(&b[..])?;
                return Ok(PublicKey::Ed25519(key));
            }
            #[cfg(feature = "secp256k1")]
            if let Sum::Secp256k1(b) = sum {
                return Self::from_raw_secp256k1(b)
                    .ok_or_else(|| Error::invalid_key("malformed key".to_string()));
            }
            Err(Error::invalid_key("not an ed25519 key".to_string()))
        }
    }

    impl From<PublicKey> for RawPublicKey {
        fn from(value: PublicKey) -> Self {
            match value {
                PublicKey::Ed25519(ref pk) => RawPublicKey {
                    sum: Some(Sum::Ed25519(
                        pk.as_bytes().to_vec(),
                    )),
                },
                #[cfg(feature = "secp256k1")]
                PublicKey::Secp256k1(ref pk) => RawPublicKey {
                    sum: Some(Sum::Secp256k1(
                        pk.to_sec1_bytes().into(),
                    )),
                },
            }
        }
    }
}

impl PublicKey {
    /// From raw secp256k1 public key bytes
    #[cfg(feature = "secp256k1")]
    #[cfg_attr(docsrs, doc(cfg(feature = "secp256k1")))]
    pub fn from_raw_secp256k1(bytes: &[u8]) -> Option<PublicKey> {
        Secp256k1::from_sec1_bytes(bytes)
            .ok()
            .map(PublicKey::Secp256k1)
    }

    /// From raw Ed25519 public key bytes
    pub fn from_raw_ed25519(bytes: &[u8]) -> Option<PublicKey> {
        Ed25519::try_from(bytes).map(PublicKey::Ed25519).ok()
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

    /// Serialize this key as a byte vector.
    pub fn to_bytes(self) -> Vec<u8> {
        match self {
            PublicKey::Ed25519(pk) => pk.as_bytes().to_vec(),
            #[cfg(feature = "secp256k1")]
            PublicKey::Secp256k1(pk) => pk.to_sec1_bytes().into(),
        }
    }

    /// Serialize this key as Bech32 with the given human readable prefix
    pub fn to_bech32(self, hrp: &str) -> String {
        let backward_compatible_amino_prefixed_pubkey = match self {
            PublicKey::Ed25519(ref pk) => {
                let mut key_bytes = vec![0x16, 0x24, 0xDE, 0x64, 0x20];
                key_bytes.extend(pk.as_bytes());
                key_bytes
            },
            #[cfg(feature = "secp256k1")]
            PublicKey::Secp256k1(ref pk) => {
                let mut key_bytes = vec![0xEB, 0x5A, 0xE9, 0x87, 0x21];
                key_bytes.extend(pk.to_sec1_bytes().as_ref());
                key_bytes
            },
        };
        bech32::encode(hrp, backward_compatible_amino_prefixed_pubkey)
    }

    /// Serialize this key as hexadecimal
    pub fn to_hex(self) -> String {
        String::from_utf8(hex::encode_upper(self.to_bytes())).unwrap()
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
                PublicKey::Secp256k1(b) => a.cmp(b),
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
            #[cfg(feature = "secp256k1")]
            PublicKey::Secp256k1(_) => Ok(TendermintKey::AccountKey(public_key)),

            _ => Err(Error::invalid_key(
                "only ed25519 or secp256k1 consensus keys are supported".to_string(),
            )),
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
            _ => Err(Error::parse(format!("invalid algorithm: {s}"))),
        }
    }
}

impl Serialize for Algorithm {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.as_str().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Algorithm {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
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
    String::from_utf8(base64::encode(pk.to_sec1_bytes()))
        .unwrap()
        .serialize(serializer)
}

fn deserialize_ed25519_base64<'de, D>(deserializer: D) -> Result<Ed25519, D::Error>
where
    D: Deserializer<'de>,
{
    use de::Error;
    let encoded = String::deserialize(deserializer)?;
    let bytes = base64::decode(encoded).map_err(D::Error::custom)?;
    Ed25519::try_from(&bytes[..]).map_err(|_| D::Error::custom("invalid Ed25519 key"))
}

#[cfg(feature = "secp256k1")]
fn deserialize_secp256k1_base64<'de, D>(deserializer: D) -> Result<Secp256k1, D::Error>
where
    D: Deserializer<'de>,
{
    use de::Error;
    let encoded = String::deserialize(deserializer)?;
    let bytes = base64::decode(encoded).map_err(D::Error::custom)?;
    Secp256k1::from_sec1_bytes(&bytes).map_err(|_| D::Error::custom("invalid secp256k1 key"))
}

#[cfg(test)]
mod tests {
    use subtle_encoding::hex;

    use super::{PublicKey, TendermintKey};
    use crate::{prelude::*, public_key::PubKeyResponse};

    const EXAMPLE_CONSENSUS_KEY: &str =
        "4A25C6640A1F72B9C975338294EF51B6D1C33158BB6ECBA69FBC3FB5A33C9DCE";

    #[test]
    fn test_consensus_serialization() {
        let example_key = TendermintKey::ConsensusKey(
            PublicKey::from_raw_ed25519(&hex::decode_upper(EXAMPLE_CONSENSUS_KEY).unwrap())
                .unwrap(),
        );
        // Key created from:
        // import (
        // "encoding/hex"
        // "fmt"
        // "github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
        // "github.com/cosmos/cosmos-sdk/types"
        // )
        //
        // func bech32conspub() {
        // pubBz, _ :=
        // hex.DecodeString("4A25C6640A1F72B9C975338294EF51B6D1C33158BB6ECBA69FBC3FB5A33C9DCE")
        // pub := &ed25519.PubKey{Key: pubBz}
        // mustBech32ConsPub := types.MustBech32ifyPubKey(types.Bech32PubKeyTypeConsPub, pub)
        // fmt.Println(mustBech32ConsPub)
        // }
        assert_eq!(
            example_key.public_key().to_bech32("cosmosvalconspub"),
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
            example_key.public_key().to_bech32("cosmospub"),
            "cosmospub1addwnpepq2skx090esq7h7md0r3e76r6ruyet330e904r6k3pgpwuzl92x6actrt4uq"
        );
    }

    #[test]
    fn json_parsing() {
        let json_string = "{\"type\":\"tendermint/PubKeyEd25519\",\"value\":\"RblzMO4is5L1hZz6wo4kPbptzOyue6LTk4+lPhD1FRk=\"}";
        let pubkey: PublicKey = serde_json::from_str(json_string).unwrap();

        assert_eq!(
            pubkey.ed25519().unwrap().as_bytes(),
            [
                69, 185, 115, 48, 238, 34, 179, 146, 245, 133, 156, 250, 194, 142, 36, 61, 186,
                109, 204, 236, 174, 123, 162, 211, 147, 143, 165, 62, 16, 245, 21, 25
            ]
        );

        let reserialized_json = serde_json::to_string(&pubkey).unwrap();
        assert_eq!(reserialized_json.as_str(), json_string);
    }

    tendermint_pb_modules! {
        use super::*;
        use pb::privval::PubKeyResponse as RawPubKeyResponse;

        #[test]
        fn test_ed25519_pubkey_msg() {
            // test-vector generated from Go
            // import (
            // "fmt"
            // "github.com/tendermint/tendermint/proto/tendermint/crypto"
            // "github.com/tendermint/tendermint/proto/tendermint/privval"
            // )
            //
            // func ed25519_key() {
            // pkr := &privval.PubKeyResponse{
            // PubKey: &crypto.PublicKey{
            // Sum: &crypto.PublicKey_Ed25519{Ed25519: []byte{
            // 215, 90, 152, 1, 130, 177, 10, 183, 213, 75, 254, 211, 201, 100, 7, 58,
            // 14, 225, 114, 243, 218, 166, 35, 37, 175, 2, 26, 104, 247, 7, 81, 26,
            // },
            // },
            // },
            // Error: nil,
            // }
            // pbpk, _ := pkr.Marshal()
            // fmt.Printf("%#v\n", pbpk)
            //
            // }
            let encoded = vec![
                0xa, 0x22, 0xa, 0x20, 0xd7, 0x5a, 0x98, 0x1, 0x82, 0xb1, 0xa, 0xb7, 0xd5, 0x4b, 0xfe,
                0xd3, 0xc9, 0x64, 0x7, 0x3a, 0xe, 0xe1, 0x72, 0xf3, 0xda, 0xa6, 0x23, 0x25, 0xaf, 0x2,
                0x1a, 0x68, 0xf7, 0x7, 0x51, 0x1a,
            ];

            let msg = PubKeyResponse {
                pub_key: Some(
                    PublicKey::from_raw_ed25519(&[
                        215, 90, 152, 1, 130, 177, 10, 183, 213, 75, 254, 211, 201, 100, 7, 58, 14,
                        225, 114, 243, 218, 166, 35, 37, 175, 2, 26, 104, 247, 7, 81, 26,
                    ])
                    .unwrap(),
                ),
                error: None,
            };
            let got = Protobuf::<RawPubKeyResponse>::encode_vec(msg.clone());

            assert_eq!(got, encoded);
            let decoded = <PubKeyResponse as Protobuf<RawPubKeyResponse>>::decode_vec(
                &encoded
            ).unwrap();
            assert_eq!(decoded, msg);
        }
    }
}
