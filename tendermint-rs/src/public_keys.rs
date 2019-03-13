//! Public keys used in Tendermint networks

use crate::{amino_types::PubKeyResponse, error::Error};
use signatory::{ecdsa::curve::secp256k1, ed25519};
use std::ops::Deref;
use subtle_encoding::{bech32, hex};

/// Public keys allowed in Tendermint protocols
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub enum PublicKey {
    /// Ed25519 keys
    Ed25519(ed25519::PublicKey),

    /// Secp256k1 keys
    Secp256k1(secp256k1::PublicKey),
}

impl PublicKey {
    /// From raw secp256k1 public key bytes
    pub fn from_raw_secp256k1(bytes: &[u8]) -> Result<PublicKey, Error> {
        Ok(PublicKey::Secp256k1(secp256k1::PublicKey::from_bytes(
            bytes,
        )?))
    }

    /// From raw Ed25519 public key bytes
    pub fn from_raw_ed25519(bytes: &[u8]) -> Result<PublicKey, Error> {
        Ok(PublicKey::Ed25519(ed25519::PublicKey::from_bytes(bytes)?))
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

    /// Create a response which represents this public key
    pub fn to_response(self) -> PubKeyResponse {
        match self {
            PublicKey::Ed25519(ref pk) => PubKeyResponse {
                pub_key_ed25519: pk.as_bytes().to_vec(),
            },
            PublicKey::Secp256k1(_) => panic!("secp256k1 PubKeyResponse unimplemented"),
        }
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

impl Deref for TendermintKey {
    type Target = PublicKey;

    fn deref(&self) -> &PublicKey {
        match self {
            TendermintKey::AccountKey(key) => key,
            TendermintKey::ConsensusKey(key) => key,
        }
    }
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
}
