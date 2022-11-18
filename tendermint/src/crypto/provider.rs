use digest::{consts::U32, Digest, FixedOutputReset};
use signature::{Signature, Signer, Verifier};

pub trait CryptoProvider {
    type Sha256: Digest<OutputSize = U32> + FixedOutputReset;

    type EcdsaSecp256k1Signature: Signature;
    type EcdsaSecp256k1Signer: Signer<Self::EcdsaSecp256k1Signature>;
    type EcdsaSecp256k1Verifier: Verifier<Self::EcdsaSecp256k1Signature>;
}

#[cfg(test)]
mod tests {

    /// A draft for an imlpementation of the HostFunctionManager for a specific chain (i.e. Polkadot/Substrate)
    /// that uses the [`CryptoProvider`] trait
    use core::marker::PhantomData;
    use signature::{DigestSigner, DigestVerifier};

    use digest::{consts::U32, Digest, FixedOutput, FixedOutputReset, Reset};
    use signature::{Signature, Signer, Verifier};

    use super::CryptoProvider;

    struct SubstrateHostFunctionsManager;
    use k256::ecdsa::{SigningKey, VerifyingKey};

    #[derive(Debug, Default)]
    struct SubstrateSha256(sha2::Sha256);

    struct SubstrateSigner<D> {
        inner: SigningKey,
        _d: PhantomData<D>,
    }
    #[derive(Debug)]
    struct SubstrateSignatureVerifier<D> {
        inner: VerifyingKey,
        _d: PhantomData<D>,
    }

    impl<D: Digest + FixedOutput<OutputSize = U32>> SubstrateSignatureVerifier<D> {
        fn from_bytes(public_key: &[u8]) -> Result<Self, ed25519::Error> {
            Ok(Self {
                inner: VerifyingKey::from_sec1_bytes(public_key)?,
                _d: PhantomData::default(),
            })
        }
    }

    impl<D: Digest + FixedOutput<OutputSize = U32>, S: Signature> DigestVerifier<D, S>
        for SubstrateSignatureVerifier<D>
    where
        VerifyingKey: DigestVerifier<D, S>,
    {
        fn verify_digest(&self, digest: D, signature: &S) -> Result<(), ed25519::Error> {
            self.inner.verify_digest(digest, signature)
        }
    }

    impl<S: signature::PrehashSignature, D: Digest + FixedOutput<OutputSize = U32>> Verifier<S>
        for SubstrateSignatureVerifier<D>
    where
        VerifyingKey: DigestVerifier<D, S>,
    {
        fn verify(&self, msg: &[u8], signature: &S) -> Result<(), ed25519::Error> {
            let mut hasher = D::new();
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

    impl<D: Digest, S: Signature> Signer<S> for SubstrateSigner<D>
    where
        SigningKey: DigestSigner<D, S>,
    {
        fn try_sign(&self, msg: &[u8]) -> Result<S, ed25519::Error> {
            let mut hasher = D::new();
            Digest::update(&mut hasher, msg);
            self.inner.try_sign_digest(hasher)
        }
    }

    trait SubstrateHostFunctions: CryptoProvider {
        fn sha2_256(preimage: &[u8]) -> [u8; 32];
        fn ed25519_verify(sig: &[u8], msg: &[u8], pub_key: &[u8]) -> Result<(), ()>;
        fn secp256k1_verify(sig: &[u8], message: &[u8], public: &[u8]) -> Result<(), ()>;
    }

    impl CryptoProvider for SubstrateHostFunctionsManager {
        type Sha256 = SubstrateSha256;

        type EcdsaSecp256k1Signature = k256::ecdsa::Signature;
        type EcdsaSecp256k1Signer = SubstrateSigner<Self::Sha256>;
        type EcdsaSecp256k1Verifier = SubstrateSignatureVerifier<Self::Sha256>;
    }

    impl SubstrateHostFunctions for SubstrateHostFunctionsManager {
        fn sha2_256(preimage: &[u8]) -> [u8; 32] {
            let mut hasher = Self::Sha256::new();
            hasher.update(preimage);
            let result = hasher.finalize().try_into().unwrap();
            result
        }
        fn ed25519_verify(sig: &[u8], msg: &[u8], pub_key: &[u8]) -> Result<(), ()> {
            let verifier =
                <<Self as CryptoProvider>::EcdsaSecp256k1Verifier>::from_bytes(pub_key).unwrap();
            let signature = k256::ecdsa::Signature::from_der(sig).unwrap();
            Ok(verifier.verify(msg, &signature).unwrap())
        }

        fn secp256k1_verify(_sig: &[u8], _message: &[u8], _public: &[u8]) -> Result<(), ()> {
            unimplemented!()
        }
    }
}
