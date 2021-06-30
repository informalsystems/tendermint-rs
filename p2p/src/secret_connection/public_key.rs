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
    ///
    /// # Errors
    ///
    /// * if the bytes given are invalid
    pub fn from_raw_ed25519(bytes: &[u8]) -> Result<Self, Error> {
        ed25519::PublicKey::from_bytes(bytes)
            .map(Self::Ed25519)
            .map_err(error::signature_error)
    }

    /// Get Ed25519 public key
    #[must_use]
    pub const fn ed25519(self) -> Option<ed25519::PublicKey> {
        match self {
            Self::Ed25519(pk) => Some(pk),
        }
    }

    /// Get the remote Peer ID
    #[must_use]
    pub fn peer_id(self) -> node::Id {
        match self {
            Self::Ed25519(pk) => {
                // TODO(tarcieri): use `tendermint::node::Id::from`
                let digest = Sha256::digest(pk.as_bytes());
                let mut bytes = [0_u8; 20];
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
    fn from(sk: &ed25519::Keypair) -> Self {
        Self::Ed25519(sk.public)
    }
}

impl From<ed25519::PublicKey> for PublicKey {
    fn from(pk: ed25519::PublicKey) -> Self {
        Self::Ed25519(pk)
    }
}
