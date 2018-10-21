use iq_bech32::Bech32;
use sha2::{Digest, Sha256};
use signatory::ed25519;
pub use signatory::ed25519::PUBLIC_KEY_SIZE;
use std::fmt::{self, Display};
use subtle_encoding::hex;
#[cfg(feature = "yubihsm")]
use yubihsm;

use error::{KmsError, KmsErrorKind::*};

/// Ed25519 public keys
#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct PublicKey(ed25519::PublicKey);

impl PublicKey {
    /// Convert a bytestring to a public key
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, KmsError> {
        Ok(PublicKey(
            ed25519::PublicKey::from_bytes(bytes).map_err(|e| err!(InvalidKey, "{}", e))?,
        ))
    }

    /// Obtain public key as a byte array reference
    #[inline]
    pub fn as_bytes(&self) -> &[u8; PUBLIC_KEY_SIZE] {
        self.0.as_bytes()
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bech: Bech32 = Default::default();

        let bech_str = bech.encode("rawed25519", self.as_bytes());

        write!(f, "{}", bech_str)?;
        Ok(())
    }
}

pub struct SecretConnectionKey(pub PublicKey);
pub struct ConsensusKey(pub PublicKey);

impl Display for ConsensusKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bech: Bech32 = Default::default();

        let bech_str = bech.encode("cosmosvalconspub", self.0.as_bytes());

        write!(f, "{}", bech_str)?;
        Ok(())
    }
}

impl Display for SecretConnectionKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut sh = Sha256::default();
        sh.input(self.0.as_bytes());
        let hex_str = hex::encode(&sh.result().as_slice()[0..20]);
        write!(f, "{}", String::from_utf8(hex_str).unwrap())?;
        Ok(())
    }
}
impl From<ed25519::PublicKey> for PublicKey {
    fn from(key: ed25519::PublicKey) -> PublicKey {
        PublicKey(key)
    }
}

#[cfg(feature = "yubihsm")]
impl From<yubihsm::client::PublicKey> for PublicKey {
    fn from(key: yubihsm::client::PublicKey) -> PublicKey {
        assert_eq!(key.algorithm, yubihsm::AsymmetricAlg::Ed25519);
        Self::from_bytes(key.as_slice()).unwrap()
    }
}
