use sha2::Sha256;

use super::CryptoProvider;

/// A batteries-included provider of cryptograpic functions.
pub struct DefaultCryptoProvider {}

impl CryptoProvider for DefaultCryptoProvider {
    type Sha256 = Sha256;

    type EcdsaSecp256k1Signature = k256::ecdsa::Signature;

    type EcdsaSecp256k1Signer = k256::ecdsa::SigningKey;

    type EcdsaSecp256k1Verifier = k256::ecdsa::VerifyingKey;
}
