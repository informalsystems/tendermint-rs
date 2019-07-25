//! Cryptographic private keys

#[cfg(feature = "signatory-dalek")]
use crate::public_key::PublicKey;
use serde::{de, de::Error as _, ser, Deserialize, Serialize};
use signatory::ed25519;
#[cfg(feature = "signatory-dalek")]
use signatory::PublicKeyed;
#[cfg(feature = "signatory-dalek")]
use signatory_dalek::Ed25519Signer;
use subtle_encoding::{Base64, Encoding};
use zeroize::{Zeroize, Zeroizing};

/// Size of an Ed25519 keypair (private + public key) in bytes
pub const ED25519_KEYPAIR_SIZE: usize = 64;

/// Private keys as parsed from configuration files
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum PrivateKey {
    /// Ed25519 keys
    #[serde(rename = "tendermint/PrivKeyEd25519")]
    Ed25519(Ed25519Keypair),
}

impl PrivateKey {
    /// Get the public key associated with this private key
    #[cfg(feature = "signatory-dalek")]
    pub fn public_key(&self) -> PublicKey {
        match self {
            PrivateKey::Ed25519(private_key) => private_key.public_key(),
        }
    }

    /// If applicable, borrow the Ed25519 keypair
    pub fn ed25519_keypair(&self) -> Option<&Ed25519Keypair> {
        match self {
            PrivateKey::Ed25519(keypair) => Some(keypair),
        }
    }
}

/// Ed25519 keypairs
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct Ed25519Keypair([u8; ED25519_KEYPAIR_SIZE]);

impl Ed25519Keypair {
    /// Get the public key associated with this keypair
    #[cfg(feature = "signatory-dalek")]
    pub fn public_key(&self) -> PublicKey {
        let seed = ed25519::Seed::from_keypair(&self.0[..]).unwrap();
        let pk = signatory_dalek::Ed25519Signer::from(&seed)
            .public_key()
            .unwrap();

        PublicKey::from(pk)
    }

    /// Get the Signatory Ed25519 "seed" for this signer
    pub fn to_seed(&self) -> ed25519::Seed {
        ed25519::Seed::from(self)
    }

    /// Get a Signatory Ed25519 signer (ed25519-dalek based)
    #[cfg(feature = "signatory-dalek")]
    pub fn to_signer(&self) -> Ed25519Signer {
        Ed25519Signer::from(&self.to_seed())
    }
}

impl<'a> From<&'a Ed25519Keypair> for ed25519::Seed {
    fn from(keypair: &'a Ed25519Keypair) -> ed25519::Seed {
        ed25519::Seed::from_keypair(&keypair.0[..]).unwrap()
    }
}

impl<'de> Deserialize<'de> for Ed25519Keypair {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let string = Zeroizing::new(String::deserialize(deserializer)?);

        let mut keypair_bytes = [0u8; ED25519_KEYPAIR_SIZE];
        let decoded_len = Base64::default()
            .decode_to_slice(string.as_bytes(), &mut keypair_bytes)
            .map_err(|_| D::Error::custom("invalid Ed25519 keypair"))?;

        if decoded_len != ED25519_KEYPAIR_SIZE {
            return Err(D::Error::custom("invalid Ed25519 keypair size"));
        }

        Ok(Ed25519Keypair(keypair_bytes))
    }
}

impl Serialize for Ed25519Keypair {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        String::from_utf8(Base64::default().encode(&self.0[..]))
            .unwrap()
            .serialize(serializer)
    }
}
