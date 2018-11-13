//! Public keys used in Tendermint networks
// TODO:: account keys

use sha2::{Digest, Sha256};
use signatory::ed25519;
use std::fmt::{self, Display};
use subtle_encoding::bech32;

use error::Error;

/// Validator signing keys used for authenticating consensus protocol messages
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum ConsensusKey {
    /// Ed25519 consensus keys
    Ed25519(ed25519::PublicKey),
}

impl ConsensusKey {
    /// From raw Ed25519 public key bytes
    pub fn from_raw_ed25519(bytes: &[u8]) -> Result<ConsensusKey, Error> {
        Ok(ConsensusKey::Ed25519(ed25519::PublicKey::from_bytes(
            bytes,
        )?))
    }

    /// Get Ed25519 public key
    pub fn ed25519(self) -> Option<ed25519::PublicKey> {
        match self {
            ConsensusKey::Ed25519(pk) => Some(pk),
        }
    }
}

impl Display for ConsensusKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConsensusKey::Ed25519(ref pk) => {
                bech32::encode("cosmosvalconspub", pk.as_bytes()).fmt(f)
            }
        }
    }
}

impl From<ed25519::PublicKey> for ConsensusKey {
    fn from(pk: ed25519::PublicKey) -> ConsensusKey {
        ConsensusKey::Ed25519(pk)
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
    use super::SecretConnectionKey;
    use subtle_encoding::hex;

    const EXAMPLE_KEY: &str = "F7FEB0B5BA0760B2C58893E329475D1EA81781DD636E37144B6D599AD38AA825";

    #[test]
    fn test_address_serialization() {
        let example_key =
            SecretConnectionKey::from_raw_ed25519(&hex::decode_upper(EXAMPLE_KEY).unwrap())
                .unwrap();

        assert_eq!(
            example_key.to_string(),
            "117C95C4FD7E636C38D303493302D2C271A39669"
        );
    }
}
