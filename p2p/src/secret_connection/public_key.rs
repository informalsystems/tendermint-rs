//! Secret Connection peer public keys

use ed25519_dalek as ed25519;
use sha2::{digest::Digest, Sha256};
use std::fmt::{self, Display};
use tendermint::{
    error::{self, Error},
    node,
};

/// Secret Connection peer public keys (signing, presently Ed25519-only)
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PublicKey {
    /// Ed25519 Secret Connection Keys
    Ed25519(ed25519::PublicKey),
}

impl PublicKey {
    /// From raw Ed25519 public key bytes
    pub fn from_raw_ed25519(bytes: &[u8]) -> Result<PublicKey, Error> {
        ed25519::PublicKey::from_bytes(bytes)
            .map(PublicKey::Ed25519)
            .map_err(|_| error::Kind::Crypto.into())
    }

    /// Get Ed25519 public key
    pub fn ed25519(self) -> Option<ed25519::PublicKey> {
        match self {
            PublicKey::Ed25519(pk) => Some(pk),
        }
    }

    /// Get the remote Peer ID
    pub fn peer_id(self) -> node::Id {
        match self {
            PublicKey::Ed25519(pk) => {
                // TODO(tarcieri): use `tendermint::node::Id::from`
                let digest = Sha256::digest(pk.as_bytes());
                let mut bytes = [0u8; 20];
                bytes.copy_from_slice(&digest[..20]);
                node::Id::new(bytes)
            }
        }
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.peer_id())
    }
}

impl From<&ed25519::Keypair> for PublicKey {
    fn from(sk: &ed25519::Keypair) -> PublicKey {
        PublicKey::Ed25519(sk.public)
    }
}

impl From<ed25519::PublicKey> for PublicKey {
    fn from(pk: ed25519::PublicKey) -> PublicKey {
        PublicKey::Ed25519(pk)
    }
}

#[cfg(test)]
mod tests {
    use super::PublicKey;
    use subtle_encoding::hex;

    const EXAMPLE_SECRET_CONN_KEY: &str =
        "F7FEB0B5BA0760B2C58893E329475D1EA81781DD636E37144B6D599AD38AA825";

    #[test]
    fn test_secret_connection_pubkey_serialization() {
        let example_key =
            PublicKey::from_raw_ed25519(&hex::decode_upper(EXAMPLE_SECRET_CONN_KEY).unwrap())
                .unwrap();

        assert_eq!(
            example_key.to_string(),
            "117C95C4FD7E636C38D303493302D2C271A39669"
        );
    }
}
