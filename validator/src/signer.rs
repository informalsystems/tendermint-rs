//! Signing-related interface and the sample software-only implementation.

use ed25519_consensus::SigningKey;
use rand_core::{CryptoRng, RngCore};
use tendermint::{account, PrivateKey, PublicKey, Signature};

use crate::error::Error;

/// The trait for different signing backends
/// (HSMs, TEEs, software-only, remote services etc.)
#[tonic::async_trait]
pub trait SignerProvider {
    type E: std::error::Error;
    async fn sign(&self, signable_bytes: &[u8]) -> Result<Signature, Self::E>;
    async fn load_pubkey(&self) -> Result<PublicKey, Self::E>;
}

/// A helper function that will convert the validator public key
/// into textual forms that are needed in different operational contexts
/// (genesis, sending a validator creation transaction, etc.).
/// The returned textual components are:
/// 1. "address": hexadecimal string of 20 bytes from the public key hash
/// 2. "public key": a json-encoded representation of the public key type
///                  and its value as a base64-encoded string
pub fn display_validator_info(pubkey: &PublicKey) -> (String, String) {
    let address = account::Id::from(*pubkey);
    // the `pubkey` is expected to be valid, so this shouldn't fail
    let pubkeyb64json = serde_json::to_string(pubkey).expect("pubkey to base64");
    (format!("{}", address), pubkeyb64json)
}

/// The default software-only implementation of [`SignerProvider`].
/// (Not recommended for production use, but it is useful for testing
/// or in combination with additional isolation from the host system, e.g. TEE.)
pub struct SoftwareSigner {
    secret_key: PrivateKey,
}

impl SoftwareSigner {
    /// The default constructor
    pub fn new(secret_key: PrivateKey) -> Self {
        Self { secret_key }
    }

    /// Generate a new random private key.
    pub fn generate_ed25519<R: RngCore + CryptoRng>(rng: R) -> Self {
        let key = SigningKey::new(rng);
        let sk = PrivateKey::Ed25519(key);
        Self::new(sk)
    }
}

#[tonic::async_trait]
impl SignerProvider for SoftwareSigner {
    type E = Error;
    async fn sign(&self, signable_bytes: &[u8]) -> Result<Signature, Error> {
        Ok(self.secret_key.sign(signable_bytes))
    }

    async fn load_pubkey(&self) -> Result<PublicKey, Error> {
        Ok(self.secret_key.public_key())
    }
}

#[cfg(test)]
mod test {
    use ed25519_consensus::SigningKey;
    use tendermint::PublicKey;

    use crate::{SignerProvider, SoftwareSigner};

    #[test]
    pub fn test_display_validator_info() {
        let signing_key = SigningKey::from([2u8; 32]);
        let ver_key = signing_key.verification_key();
        let pubkey = PublicKey::from(ver_key);
        let (address, pubkeyb64) = super::display_validator_info(&pubkey);
        assert_eq!(address, "6A3803D5F059902A1C6DAFBC9BA4729212F7CAAC");
        assert_eq!(pubkeyb64, "{\"type\":\"tendermint/PubKeyEd25519\",\"value\":\"gTl3Dqh9F19Wo1Rmw0x+zMuNipG07jeiXfYPW4/Js5Q=\"}");
    }

    #[tokio::test]
    pub async fn test_generate_sign() {
        let rng = rand_core::OsRng;
        let signer = SoftwareSigner::generate_ed25519(rng);
        let signable_bytes = b"test message";
        let signature = signer.sign(signable_bytes).await.expect("sign");
        let pubkey = signer.load_pubkey().await.expect("pubkey");
        assert!(pubkey.verify(signable_bytes, &signature).is_ok());
    }
}
