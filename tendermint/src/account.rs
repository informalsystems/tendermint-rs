//! Tendermint accounts

use core::{
    convert::{TryFrom, TryInto},
    fmt::{self, Debug, Display},
    str::FromStr,
};

use bytes::Bytes;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use subtle::{self, ConstantTimeEq};
use subtle_encoding::hex;

use tendermint_proto::Protobuf;

use crate::{error::Error, prelude::*};

/// Size of an  account ID in bytes
pub const LENGTH: usize = 20;

/// Account IDs
#[derive(Copy, Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Id([u8; LENGTH]); // JSON custom serialization for priv_validator_key.json

impl Protobuf<Vec<u8>> for Id {}

impl TryFrom<Vec<u8>> for Id {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.len() != LENGTH {
            return Err(Error::invalid_account_id_length());
        }
        let mut slice: [u8; LENGTH] = [0; LENGTH];
        slice.copy_from_slice(&value[..]);
        Ok(Id(slice))
    }
}

impl From<Id> for Vec<u8> {
    fn from(value: Id) -> Self {
        value.as_bytes().to_vec()
    }
}

impl TryFrom<Bytes> for Id {
    type Error = Error;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        if value.len() != LENGTH {
            return Err(Error::invalid_account_id_length());
        }
        let mut slice: [u8; LENGTH] = [0; LENGTH];
        slice.copy_from_slice(&value[..]);
        Ok(Id(slice))
    }
}

impl From<Id> for Bytes {
    fn from(value: Id) -> Self {
        Bytes::copy_from_slice(value.as_bytes())
    }
}

impl Id {
    /// Create a new account ID from raw bytes
    pub fn new(bytes: [u8; LENGTH]) -> Id {
        Id(bytes)
    }

    /// Borrow the account ID as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.0[..]
    }
}

impl AsRef<[u8]> for Id {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl ConstantTimeEq for Id {
    #[inline]
    fn ct_eq(&self, other: &Id) -> subtle::Choice {
        self.as_bytes().ct_eq(other.as_bytes())
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{byte:02X}")?;
        }
        Ok(())
    }
}

impl Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "account::Id({self})")
    }
}

#[cfg(feature = "rust-crypto")]
mod key_conversions {
    use super::{Id, LENGTH};
    use crate::crypto::default::Sha256;
    #[cfg(feature = "secp256k1")]
    use crate::public_key::Secp256k1;
    use crate::public_key::{Ed25519, PublicKey};
    use digest::Digest;

    // RIPEMD160(SHA256(pk))
    #[cfg(feature = "secp256k1")]
    impl From<Secp256k1> for Id {
        fn from(pk: Secp256k1) -> Id {
            use ripemd::Ripemd160;

            let sha_digest = Sha256::digest(pk.to_sec1_bytes());
            let ripemd_digest = Ripemd160::digest(&sha_digest[..]);
            let mut bytes = [0u8; LENGTH];
            bytes.copy_from_slice(&ripemd_digest[..LENGTH]);
            Id(bytes)
        }
    }

    // SHA256(pk)[:20]
    impl From<Ed25519> for Id {
        fn from(pk: Ed25519) -> Id {
            let digest = Sha256::digest(pk.as_bytes());
            Id(digest[..LENGTH].try_into().unwrap())
        }
    }

    impl From<PublicKey> for Id {
        fn from(pub_key: PublicKey) -> Id {
            match pub_key {
                PublicKey::Ed25519(pk) => Id::from(pk),
                #[cfg(feature = "secp256k1")]
                PublicKey::Secp256k1(pk) => Id::from(pk),
            }
        }
    }
}

/// Decode account ID from hex
impl FromStr for Id {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Accept either upper or lower case hex
        let bytes = hex::decode_upper(s)
            .or_else(|_| hex::decode(s))
            .map_err(Error::subtle_encoding)?;

        bytes.try_into()
    }
}

// Todo: Can I remove custom serialization?
impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(|_| {
            de::Error::custom(format!(
                "expected {}-character hex string, got {:?}",
                LENGTH * 2,
                s
            ))
        })
    }
}

impl Serialize for Id {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(
            &String::from_utf8(hex::encode_upper(Vec::<u8>::from(*self)))
                .map_err(serde::ser::Error::custom)?,
        )
    }
}

#[cfg(all(test, feature = "rust-crypto"))]
mod tests {
    use super::*;
    use crate::public_key::Ed25519;

    #[test]
    fn test_ed25519_id() {
        // test vector for pubkey and id (address)
        let pubkey_hex = "14253D61EF42D166D02E68D540D07FDF8D65A9AF0ACAA46302688E788A8521E2";
        let id_hex = "0CDA3F47EF3C4906693B170EF650EB968C5F4B2C";

        // decode pubkey and address
        let pubkey_bytes = &hex::decode_upper(pubkey_hex).unwrap();
        let id_bytes = Id::from_str(id_hex).expect("expected id_hex to decode properly");

        // get id for pubkey
        let pubkey = Ed25519::try_from(&pubkey_bytes[..]).unwrap();
        let id = Id::from(pubkey);

        assert_eq!(id_bytes.ct_eq(&id).unwrap_u8(), 1);
    }

    #[test]
    #[cfg(feature = "secp256k1")]
    fn test_secp_id() {
        use crate::public_key::Secp256k1;

        // test vector for pubkey and id (address)
        let pubkey_hex = "02950E1CDFCB133D6024109FD489F734EEB4502418E538C28481F22BCE276F248C";
        // SHA256: 034f706ac824dbb0d227c2ca30439e5be3766cfddc90f00bd530951d638b43a4
        let id_hex = "7C2BB42A8BE69791EC763E51F5A49BCD41E82237";

        // decode pubkey and address
        let pubkey_bytes = &hex::decode_upper(pubkey_hex).unwrap();
        let id_bytes = Id::from_str(id_hex).expect("expected id_hex to decode properly");

        // get id for pubkey
        let pubkey = Secp256k1::from_sec1_bytes(pubkey_bytes).unwrap();
        let id = Id::from(pubkey);

        assert_eq!(id_bytes.ct_eq(&id).unwrap_u8(), 1);
    }
}
