//! An imitation of alternative cryptographic function implementations
//! for a chain environment that provides its own cryptographic API.
#![cfg(all(feature = "secp256k1", feature = "rust-crypto"))]

use ::signature::DigestVerifier;
use digest::Digest;

use tendermint::crypto::signature::{self, Verifier};
use tendermint::crypto::{sha256::HASH_SIZE, Sha256};
use tendermint::{PublicKey, Signature};

#[derive(Debug, Default)]
struct SubstrateSha256(sha2::Sha256);

#[derive(Debug, Default)]
struct SubstrateSignatureVerifier;

impl Verifier for SubstrateSignatureVerifier {
    fn verify(
        pubkey: PublicKey,
        msg: &[u8],
        signature: &Signature,
    ) -> Result<(), signature::Error> {
        match pubkey {
            PublicKey::Secp256k1(pk) => {
                let sig = k256::ecdsa::Signature::try_from(signature.as_bytes())
                    .map_err(|_| signature::Error::MalformedSignature)?;
                let mut hasher = sha2::Sha256::new();
                Digest::update(&mut hasher, msg);
                pk.verify_digest(hasher, &sig)
                    .map_err(|_| signature::Error::VerificationFailed)
            },
            _ => Err(signature::Error::UnsupportedKeyType),
        }
    }
}

impl Sha256 for SubstrateSha256 {
    fn digest(data: impl AsRef<[u8]>) -> [u8; HASH_SIZE] {
        <sha2::Sha256 as Sha256>::digest(data)
    }
}

mod tests {
    use super::{SubstrateSha256, SubstrateSignatureVerifier};
    use tendermint::crypto::signature::Verifier;
    use tendermint::crypto::Sha256;
    use tendermint::{PublicKey, Signature};

    use subtle_encoding::hex;

    #[test]
    fn sha256_can_hash() {
        let hash = SubstrateSha256::digest(b"hello world");

        let hash = String::from_utf8(hex::encode(hash)).unwrap();
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    //const SIGNING_KEY: &[u8] = b"59820654790d53a23d1017b50ddcdb31242e27c682a0a1372fc63c01dd48816a";
    const VERIFYING_KEY: &[u8] =
        b"03cf7a110053a95b4b25266c3416ae342eba2ca3f4658fa1069fcf750f760b8c42";
    const MESSAGE: &[u8] = b"hello world";
    const SIGNATURE: &str = "684c3c183f76a79fc116dd4edd39fe40737cea51c6c1df47ff544c20d14a7a76754c43c51e0daa647e8e4164f254bb62dbf9bd5b2e2e03ffb8247dd92ce1e1e3";

    #[test]
    fn verifier_can_verify() {
        let key_bytes = hex::decode(VERIFYING_KEY).unwrap();
        let public_key = PublicKey::from_raw_secp256k1(&key_bytes).unwrap();
        let sig_bytes = hex::decode(SIGNATURE.as_bytes()).unwrap();
        let signature = Signature::try_from(&sig_bytes[..]).unwrap();

        SubstrateSignatureVerifier::verify(public_key, MESSAGE, &signature).unwrap();
    }
}
