//! Secret Connection peer public keys

use std::fmt::{self, Display};

use sha2::{digest::Digest, Sha256};
use tendermint::{error::Error, node};

/// Secret Connection peer public keys (signing, presently Ed25519-only)
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PublicKey {
    /// Ed25519 Secret Connection Keys
    Ed25519(ed25519_consensus::VerificationKey),
}

impl PublicKey {
    /// From raw Ed25519 public key bytes
    ///
    /// # Errors
    ///
    /// * if the bytes given are invalid
    pub fn from_raw_ed25519(bytes: &[u8]) -> Result<Self, Error> {
        ed25519_consensus::VerificationKey::try_from(bytes)
            .map(Self::Ed25519)
            .map_err(|_| Error::signature())
    }

    /// Get Ed25519 public key
    #[must_use]
    pub const fn ed25519(self) -> Option<ed25519_consensus::VerificationKey> {
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
            },
        }
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.peer_id())
    }
}

impl From<&ed25519_consensus::SigningKey> for PublicKey {
    fn from(sk: &ed25519_consensus::SigningKey) -> Self {
        Self::Ed25519(sk.verification_key())
    }
}

impl From<ed25519_consensus::VerificationKey> for PublicKey {
    fn from(pk: ed25519_consensus::VerificationKey) -> Self {
        Self::Ed25519(pk)
    }
}
