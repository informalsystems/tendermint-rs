//! The pure Rust implementation of signature verification functions.

use crate::crypto::signature::Error;
use crate::{PublicKey, Signature};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Verifier;

impl crate::crypto::signature::Verifier for Verifier {
    fn verify(pubkey: PublicKey, msg: &[u8], signature: &Signature) -> Result<(), Error> {
        #[allow(unreachable_patterns)]
        match pubkey {
            PublicKey::Ed25519(pk) => {
                let pubkey = ed25519_consensus::VerificationKey::try_from(pk)
                    .map_err(|_| Error::MalformedPublicKey)?;
                let sig = ed25519_consensus::Signature::try_from(signature.as_bytes())
                    .map_err(|_| Error::MalformedSignature)?;
                pubkey
                    .verify(&sig, msg)
                    .map_err(|_| Error::VerificationFailed)
            },
            #[cfg(feature = "secp256k1")]
            PublicKey::Secp256k1(pk) => {
                let signature = k256::ecdsa::Signature::try_from(signature.as_bytes())
                    .map_err(|_| Error::MalformedSignature)?;
                pk.verify(msg, &sig).map_err(|_| Error::VerificationFailed)
            },
            _ => Err(Error::UnsupportedKeyType),
        }
    }
}
