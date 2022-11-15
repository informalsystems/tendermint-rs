use digest::{consts::U32, Digest, FixedOutput};
use signature::{DigestSigner, DigestVerifier, Signature, Signer};

use crate::signature::Verifier;

pub trait CryptoProvider {
    type S: Signature;

    type Sha256: Digest + FixedOutput<OutputSize = U32>;

    type EcdsaSecp256k1Signer: Signer<Self::S>;
    type EcdsaSecp256k1Verifier: Verifier<Self::S>;
}

/// A default implementation of the HostFunctionManager that uses the [`CryptoProvider`] trait
use core::marker::PhantomData;

pub struct DefaultHostFunctionsManager;
use k256::ecdsa::{SigningKey, VerifyingKey};

#[derive(Debug, Default)]
pub struct DefaultSha256(sha2::Sha256);

pub struct DefaultSigner<D> {
    inner: SigningKey,
    _d: PhantomData<D>,
}
#[derive(Debug)]
pub struct DefaultSignatureVerifier<D> {
    inner: VerifyingKey,
    _d: PhantomData<D>,
}

impl<D: Digest + FixedOutput<OutputSize = U32>, S: Signature> DigestVerifier<D, S>
    for DefaultSignatureVerifier<D>
where
    VerifyingKey: DigestVerifier<D, S>,
{
    fn verify_digest(&self, digest: D, signature: &S) -> Result<(), ed25519::Error> {
        self.inner.verify_digest(digest, signature)
    }
}

impl<S: signature::PrehashSignature, D: Digest + FixedOutput<OutputSize = U32>> Verifier<S>
    for DefaultSignatureVerifier<D>
where
    VerifyingKey: DigestVerifier<D, S>,
{
    fn verify(&self, msg: &[u8], signature: &S) -> Result<(), ed25519::Error> {
        let mut hasher = D::new();
        Digest::update(&mut hasher, msg);
        self.verify_digest(hasher, signature)
    }
}

impl digest::OutputSizeUser for DefaultSha256 {
    type OutputSize = U32;
}

impl digest::HashMarker for DefaultSha256 {}

impl digest::Update for DefaultSha256 {
    fn update(&mut self, data: &[u8]) {
        use sha2::Digest;
        self.0.update(data);
    }
}

impl FixedOutput for DefaultSha256 {
    fn finalize_into(self, out: &mut digest::Output<Self>) {
        use sha2::Digest;
        *out = self.0.finalize();
    }
}

impl<D: Digest, S: Signature> Signer<S> for DefaultSigner<D>
where
    SigningKey: DigestSigner<D, S>,
{
    fn try_sign(&self, msg: &[u8]) -> Result<S, ed25519::Error> {
        let mut hasher = D::new();
        Digest::update(&mut hasher, msg);
        self.inner.try_sign_digest(hasher)
    }
}

impl CryptoProvider for DefaultHostFunctionsManager {
    type S = k256::ecdsa::Signature;
    type Sha256 = DefaultSha256;

    type EcdsaSecp256k1Signer = DefaultSigner<Self::Sha256>;
    type EcdsaSecp256k1Verifier = DefaultSignatureVerifier<Self::Sha256>;
}
