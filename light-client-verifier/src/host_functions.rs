use digest::FixedOutput;
use digest::{consts::U32, Digest};
use tendermint::signature::Verifier;

pub trait CryptoProvider {
    type Sha256: Digest + FixedOutput<OutputSize = U32>;

    // type EcdsaSecp256k1Signer: Signer<k256::ecdsa::Signature>;
    type EcdsaSecp256k1Verifier: Verifier<k256::ecdsa::Signature>;

    // type Ed25519Signer: Signer<ed25519::Signature>;
    // type Ed25519Verifier: Verifier<ed25519::Signature>;
}

#[cfg(test)]
mod tests {

    use core::marker::PhantomData;

    use signature::DigestVerifier;

    use super::*;
    struct SubstrateHostFunctionsManager;
    use k256::ecdsa::VerifyingKey;

    #[derive(Debug, Default)]
    struct SubstrateSha256(sha2::Sha256);
    #[derive(Debug)]
    struct SubstrateSignatureVerifier<D> {
        inner: k256::ecdsa::VerifyingKey,
        _d: PhantomData<D>,
    }

    impl<D: Digest + FixedOutput<OutputSize = U32>> SubstrateSignatureVerifier<D> {
        fn from_bytes(public_key: &[u8]) -> Result<Self, ed25519::Error> {
            Ok(Self {
                inner: k256::ecdsa::VerifyingKey::from_sec1_bytes(public_key)?,
                _d: PhantomData::default(),
            })
        }
    }

    impl<D: Digest + FixedOutput<OutputSize = U32>, S: signature::Signature> DigestVerifier<D, S>
        for SubstrateSignatureVerifier<D>
    where
        VerifyingKey: DigestVerifier<D, S>,
    {
        fn verify_digest(&self, digest: D, signature: &S) -> Result<(), ed25519::Error> {
            self.inner.verify_digest(digest, signature)
        }
    }

    impl<S: signature::PrehashSignature, D: Digest + FixedOutput<OutputSize = U32>>
        tendermint::signature::Verifier<S> for SubstrateSignatureVerifier<D>
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
            use sha2::Digest;
            self.0.update(data);
        }
    }

    impl digest::FixedOutput for SubstrateSha256 {
        fn finalize_into(self, out: &mut digest::Output<Self>) {
            use sha2::Digest;
            *out = self.0.finalize();
        }
    }

    trait SubstrateHostFunctions: CryptoProvider {
        fn sha2_256(preimage: &[u8]) -> [u8; 32];
        fn ed25519_verify(sig: &[u8], msg: &[u8], pub_key: &[u8]) -> Result<(), ()>;
        fn secp256k1_verify(sig: &[u8], message: &[u8], public: &[u8]) -> Result<(), ()>;
    }

    impl CryptoProvider for SubstrateHostFunctionsManager {
        type Sha256 = SubstrateSha256;

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

        fn secp256k1_verify(sig: &[u8], message: &[u8], public: &[u8]) -> Result<(), ()> {
            unimplemented!()
        }
    }
}
