use digest::FixedOutput;
use digest::{consts::U32, Digest};
use tendermint::signature::Signer;
use tendermint::signature::Verifier;

pub trait CryptoProvider {
    type Sha256: Digest + FixedOutput<OutputSize = U32>;

    // type EcdsaSecp256k1Signer: Signer<k256::ecdsa::Signature>;
    // type EcdsaSecp256k1Verifier: Verifier<k256::ecdsa::Signature>;

    // type Ed25519Signer: Signer<ed25519::Signature>;
    // type Ed25519Verifier: Verifier<ed25519::Signature>;
}

#[cfg(test)]
mod tests {

    use super::*;
    struct SubstrateHostFunctionsManager;

    trait SubstrateHostFunctions: CryptoProvider {
        fn sha2_256(preimage: &[u8]) -> [u8; 32];
        fn ed25519_verify(sig: &[u8], msg: &[u8], pub_key: &[u8]) -> Result<(), ()>;
        fn secp256k1_verify(sig: &[u8], message: &[u8], public: &[u8]) -> Result<(), ()>;
    }

    impl SubstrateHostFunctionsManager for SubstrateHostFunctions {
        type Sha256 = sha2::Sha256;

        fn sha2_256(preimage: &[u8]) -> [u8; 32] {
            unimplemented!()
        }
        fn ed25519_verify(sig: &[u8], msg: &[u8], pub_key: &[u8]) -> Result<(), ()> {
            unimplemented!()
        }
        fn secp256k1_verify(sig: &[u8], message: &[u8], public: &[u8]) -> Result<(), ()> {
            unimplemented!()
        }
    }

    // impl CryptoProvider for CryptoManager {
    //     fn sha2_256(preimage: &[u8]) -> [u8; 32] {
    //         sp_core::hashing::sha2_256(preimage)
    //     }

    //     fn ed25519_verify(sig: &[u8], msg: &[u8], pub_key: &[u8]) -> Result<(), ()> {
    //         use sp_core::{ed25519, ByteArray, Pair};

    //         let signature = ed25519::Signature::from_slice(sig).ok_or(())?;

    //         let public_key = ed25519::Public::from_slice(pub_key).map_err(|_| ())?;
    //         if ed25519::Pair::verify(&signature, msg, &public_key) {
    //             return Ok(());
    //         }
    //         Err(())
    //     }

    //     fn secp256k1_verify(sig: &[u8], message: &[u8], public: &[u8]) -> Result<(), ()> {
    //         use sp_core::{ecdsa, ByteArray, Pair};

    //         let public = ecdsa::Public::from_slice(public).map_err(|_| ())?;
    //         if ecdsa::Pair::verify_weak(&sig, message, &public) {
    //             return Ok(());
    //         }

    //         Err(())
    //     }
    // }
}
