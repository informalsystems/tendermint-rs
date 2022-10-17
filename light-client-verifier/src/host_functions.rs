use digest::FixedOutput;
use digest::{consts::U32, Digest};
use tendermint::signature::Signer;
use tendermint::signature::Verifier;

pub trait CryptoProvider {
    type Sha256: Digest + FixedOutput<OutputSize = U32>;

    type EcdsaSecp256k1Signer: Signer<k256::ecdsa::Signature>;
    type EcdsaSecp256k1Verifier: Verifier<k256::ecdsa::Signature>;

    type Ed25519Signer: Signer<ed25519::Signature>;
    type Ed25519Verifier: Verifier<ed25519::Signature>;
}
