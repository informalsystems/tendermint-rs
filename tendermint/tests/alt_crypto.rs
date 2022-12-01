//! An imitation of alternative cryptographic function implementations
//! for a chain environment that provides its own cryptographic API.

use digest::{consts::U32, Digest, FixedOutput, FixedOutputReset, Reset};
use signature::{self, DigestSigner, DigestVerifier, Signature, Signer, Verifier};

use k256::ecdsa::{SigningKey, VerifyingKey};

#[derive(Debug, Default)]
struct SubstrateSha256(sha2::Sha256);

#[derive(Debug)]
struct SubstrateSignature(k256::ecdsa::Signature);

impl AsRef<[u8]> for SubstrateSignature {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl Signature for SubstrateSignature {
    fn from_bytes(bytes: &[u8]) -> Result<Self, signature::Error> {
        let inner = k256::ecdsa::Signature::from_bytes(bytes)?;
        Ok(Self(inner))
    }
}

struct SubstrateSigner {
    inner: SigningKey,
}

impl SubstrateSigner {
    fn from_bytes(private_key: &[u8]) -> Result<Self, signature::Error> {
        let inner = SigningKey::from_bytes(private_key)?;
        Ok(Self { inner })
    }
}

impl Signer<SubstrateSignature> for SubstrateSigner {
    fn try_sign(&self, msg: &[u8]) -> Result<SubstrateSignature, signature::Error> {
        let mut hasher = SubstrateSha256::new();
        hasher.update(msg);
        let signature = self.inner.try_sign_digest(hasher)?;
        Ok(SubstrateSignature(signature))
    }
}

#[derive(Debug)]
struct SubstrateSignatureVerifier {
    inner: VerifyingKey,
}

impl SubstrateSignatureVerifier {
    fn from_bytes(public_key: &[u8]) -> Result<Self, signature::Error> {
        Ok(Self {
            inner: VerifyingKey::from_sec1_bytes(public_key)?,
        })
    }
}

impl DigestVerifier<SubstrateSha256, SubstrateSignature> for SubstrateSignatureVerifier {
    fn verify_digest(
        &self,
        digest: SubstrateSha256,
        signature: &SubstrateSignature,
    ) -> Result<(), signature::Error> {
        self.inner.verify_digest(digest, &signature.0)
    }
}

impl Verifier<SubstrateSignature> for SubstrateSignatureVerifier {
    fn verify(&self, msg: &[u8], signature: &SubstrateSignature) -> Result<(), signature::Error> {
        let mut hasher = SubstrateSha256::new();
        Digest::update(&mut hasher, msg);
        self.verify_digest(hasher, signature)
    }
}

impl digest::OutputSizeUser for SubstrateSha256 {
    type OutputSize = U32;
}

impl digest::HashMarker for SubstrateSha256 {}

impl digest::Update for SubstrateSha256 {
    fn update(&mut self, data: &[u8]) {
        digest::Update::update(&mut self.0, data)
    }
}

impl FixedOutput for SubstrateSha256 {
    fn finalize_into(self, out: &mut digest::Output<Self>) {
        *out = self.0.finalize();
    }
}

impl Reset for SubstrateSha256 {
    fn reset(&mut self) {
        Reset::reset(&mut self.0)
    }
}

impl FixedOutputReset for SubstrateSha256 {
    fn finalize_into_reset(&mut self, out: &mut digest::Output<Self>) {
        *out = self.0.finalize_reset();
    }
}

mod tests {
    use super::{SubstrateSha256, SubstrateSignatureVerifier, SubstrateSigner};
    use signature::{Signature, Signer, Verifier};
    use tendermint::crypto::Sha256;

    use subtle_encoding::hex;

    #[test]
    fn sha256_can_hash() {
        let mut hasher = SubstrateSha256::new();
        hasher.update(b"hello world");
        let hash = hasher.finalize();

        let hash = String::from_utf8(hex::encode(&hash)).unwrap();
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    const SIGNING_KEY: &[u8] = b"59820654790d53a23d1017b50ddcdb31242e27c682a0a1372fc63c01dd48816a";
    const VERIFYING_KEY: &[u8] =
        b"03cf7a110053a95b4b25266c3416ae342eba2ca3f4658fa1069fcf750f760b8c42";
    const MESSAGE: &[u8] = b"hello world";
    const SIGNATURE: &str = "684c3c183f76a79fc116dd4edd39fe40737cea51c6c1df47ff544c20d14a7a76754c43c51e0daa647e8e4164f254bb62dbf9bd5b2e2e03ffb8247dd92ce1e1e3";

    #[test]
    fn signer_can_sign() {
        let key_bytes = hex::decode(SIGNING_KEY).unwrap();

        let signer = SubstrateSigner::from_bytes(&key_bytes).unwrap();
        let signature = signer.sign(MESSAGE);

        let sig_hex = String::from_utf8(hex::encode(signature.as_bytes())).unwrap();
        assert_eq!(sig_hex, SIGNATURE);
    }

    #[test]
    fn verifier_can_verify() {
        let key_bytes = hex::decode(VERIFYING_KEY).unwrap();
        let signature = hex::decode(SIGNATURE.as_bytes()).unwrap();
        let signature = Signature::from_bytes(&signature).unwrap();

        let verifier = SubstrateSignatureVerifier::from_bytes(&key_bytes).unwrap();
        verifier.verify(MESSAGE, &signature).unwrap();
    }
}
