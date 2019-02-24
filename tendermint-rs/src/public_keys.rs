//! Public keys used in Tendermint networks
// TODO:: account keys

use crate::error::Error;
use sha2::{Digest, Sha256};
use signatory::{ecdsa::curve::secp256k1, ed25519};
use std::fmt::{self, Display};
use subtle_encoding::bech32;

/// Validator signing keys used for authenticating consensus protocol messages
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum TendermintKey {
    /// Ed25519 consensus keys
    Ed25519(ed25519::PublicKey),
    /// Secp256k1 consensus keys
    Secp256k1(secp256k1::PublicKey),
}

/// Validator signing keys used for authenticating consensus protocol messages
pub struct ConsensusKey(TendermintKey);
/// User signing keys used for interacting with accounts in the state machine
pub struct AccountKey(TendermintKey);

impl TendermintKey {
    /// From raw secp256k1 public key bytes
    pub fn from_raw_secp256k1(bytes: &[u8]) -> Result<TendermintKey, Error> {
        Ok(TendermintKey::Secp256k1(secp256k1::PublicKey::from_bytes(
            bytes,
        )?))
    }

    /// From raw Ed25519 public key bytes
    pub fn from_raw_ed25519(bytes: &[u8]) -> Result<TendermintKey, Error> {
        Ok(TendermintKey::Ed25519(ed25519::PublicKey::from_bytes(
            bytes,
        )?))
    }

    /// Get Ed25519 public key
    pub fn ed25519(self) -> Option<ed25519::PublicKey> {
        match self {
            TendermintKey::Ed25519(pk) => Some(pk),
            _ => None,
        }
    }
}

impl Display for ConsensusKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let key_bytes: Vec<u8> = match self.0 {
            TendermintKey::Ed25519(ref pk) => {
                //Amino prefix for Pubkey
                let mut key_bytes = vec![0x16, 0x24, 0xDE, 0x64, 0x20];
                key_bytes.extend(pk.as_bytes());
                key_bytes
            }
            TendermintKey::Secp256k1(ref pk) => {
                let mut key_bytes = vec![0xEB, 0x5A, 0xE9, 0x87, 0x21];
                key_bytes.extend(pk.as_bytes());
                key_bytes
            }
        };
        bech32::encode("cosmosvalconspub", &key_bytes).fmt(f)
    }
}

impl Display for AccountKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let key_bytes: Vec<u8> = match self.0 {
            TendermintKey::Ed25519(ref pk) => {
                //Amino prefix for Pubkey
                let mut key_bytes = vec![0x16, 0x24, 0xDE, 0x64, 0x20];
                key_bytes.extend(pk.as_bytes());
                key_bytes
            }
            TendermintKey::Secp256k1(ref pk) => {
                let mut key_bytes = vec![0xEB, 0x5A, 0xE9, 0x87, 0x21];
                key_bytes.extend(pk.as_bytes());
                key_bytes
            }
        };
        bech32::encode("cosmospub", &key_bytes).fmt(f)
    }
}

impl From<ed25519::PublicKey> for TendermintKey {
    fn from(pk: ed25519::PublicKey) -> TendermintKey {
        TendermintKey::Ed25519(pk)
    }
}

impl From<secp256k1::PublicKey> for TendermintKey {
    fn from(pk: secp256k1::PublicKey) -> TendermintKey {
        TendermintKey::Secp256k1(pk)
    }
}

impl From<ed25519::PublicKey> for ConsensusKey {
    fn from(pk: ed25519::PublicKey) -> ConsensusKey {
        ConsensusKey(TendermintKey::Ed25519(pk))
    }
}

impl From<secp256k1::PublicKey> for ConsensusKey {
    fn from(pk: secp256k1::PublicKey) -> ConsensusKey {
        ConsensusKey(TendermintKey::Secp256k1(pk))
    }
}

impl From<ed25519::PublicKey> for AccountKey {
    fn from(pk: ed25519::PublicKey) -> AccountKey {
        AccountKey(TendermintKey::Ed25519(pk))
    }
}

impl From<secp256k1::PublicKey> for AccountKey {
    fn from(pk: secp256k1::PublicKey) -> AccountKey {
        AccountKey(TendermintKey::Secp256k1(pk))
    }
}

/// Secret Connection signing keys
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum SecretConnectionKey {
    /// Ed25519 Secret Connection keys
    Ed25519(ed25519::PublicKey),
}

impl SecretConnectionKey {
    /// From raw Ed25519 public key bytes
    pub fn from_raw_ed25519(bytes: &[u8]) -> Result<SecretConnectionKey, Error> {
        Ok(SecretConnectionKey::Ed25519(
            ed25519::PublicKey::from_bytes(bytes)?,
        ))
    }

    /// Get Ed25519 public key
    pub fn ed25519(self) -> Option<ed25519::PublicKey> {
        match self {
            SecretConnectionKey::Ed25519(pk) => Some(pk),
        }
    }
}

impl Display for SecretConnectionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecretConnectionKey::Ed25519(ref pk) => {
                for byte in &Sha256::digest(pk.as_bytes())[..20] {
                    write!(f, "{:02X}", byte)?;
                }
            }
        }
        Ok(())
    }
}

impl From<ed25519::PublicKey> for SecretConnectionKey {
    fn from(pk: ed25519::PublicKey) -> SecretConnectionKey {
        SecretConnectionKey::Ed25519(pk)
    }
}

#[cfg(test)]
mod tests {
    use super::{AccountKey, ConsensusKey, SecretConnectionKey, TendermintKey};
    use subtle_encoding::hex;

    const EXAMPLE_SECRET_CONN_KEY: &str =
        "F7FEB0B5BA0760B2C58893E329475D1EA81781DD636E37144B6D599AD38AA825";

    #[test]
    fn test_address_serialization() {
        let example_key = SecretConnectionKey::from_raw_ed25519(
            &hex::decode_upper(EXAMPLE_SECRET_CONN_KEY).unwrap(),
        )
        .unwrap();

        assert_eq!(
            example_key.to_string(),
            "117C95C4FD7E636C38D303493302D2C271A39669"
        );
    }

    const EXAMPLE_CONSENSUS_KEY: &str =
        "4A25C6640A1F72B9C975338294EF51B6D1C33158BB6ECBA69FBC3FB5A33C9DCE";

    #[test]
    fn test_consensus_serialization() {
        let example_key = ConsensusKey(
            TendermintKey::from_raw_ed25519(&hex::decode_upper(EXAMPLE_CONSENSUS_KEY).unwrap())
                .unwrap(),
        );

        assert_eq!(
            example_key.to_string(),
            "cosmosvalconspub1zcjduepqfgjuveq2raetnjt4xwpffm63kmguxv2chdhvhf5lhslmtgeunh8qmf7exk"
        );
    }

    const EXAMPLE_ACCOUNT_KEY: &str =
        "02A1633CAFCC01EBFB6D78E39F687A1F0995C62FC95F51EAD10A02EE0BE551B5DC";
    #[test]
    fn test_account_serialization() {
        let example_key = AccountKey(
            TendermintKey::from_raw_secp256k1(&hex::decode_upper(EXAMPLE_ACCOUNT_KEY).unwrap())
                .unwrap(),
        );

        assert_eq!(
            example_key.to_string(),
            "cosmospub1addwnpepq2skx090esq7h7md0r3e76r6ruyet330e904r6k3pgpwuzl92x6actrt4uq"
        );
    }
}
