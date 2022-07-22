//! Signing-related interface and the sample software-only implementation.

use async_signature::AsyncSigner;
use ed25519_consensus::SigningKey;
use rand_core::{CryptoRng, RngCore};
use tendermint::{account, PrivateKey, PublicKey, Signature};

/// The trait for different signing backends
/// (HSMs, TEEs, software-only, remote services etc.)
#[tonic::async_trait]
pub trait SignerProvider: AsyncSigner<Signature> {
    async fn load_pubkey(&self) -> Result<PublicKey, async_signature::Error>;
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
pub type SoftwareSigner = PrivateKey;

/// A helper to generate a new random private key.
pub fn generate_ed25519<R: RngCore + CryptoRng>(rng: R) -> SoftwareSigner {
    let key = SigningKey::new(rng);
    PrivateKey::Ed25519(key)
}

#[tonic::async_trait]
impl SignerProvider for SoftwareSigner {
    async fn load_pubkey(&self) -> Result<PublicKey, async_signature::Error> {
        Ok(self.public_key())
    }
}

#[cfg(test)]
mod test {
    use async_signature::AsyncSigner;
    use ed25519_consensus::SigningKey;
    use tendermint::PublicKey;

    use crate::{generate_ed25519, SignerProvider};

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
        let signer = generate_ed25519(rng);
        let signable_bytes = b"test message";
        let signature = signer.sign_async(signable_bytes).await.expect("sign");
        let pubkey = signer.load_pubkey().await.expect("pubkey");
        assert!(pubkey.verify(signable_bytes, &signature).is_ok());
    }
}
