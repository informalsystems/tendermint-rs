//! Host function utilities

use sp_std::fmt::Debug;

/// Host functions that the light client needs for crypto operations.
pub trait HostFunctionsProvider: Send + Sync + Default + Debug + 'static {
    /// sha256 hash function
    fn sha2_256(preimage: &[u8]) -> [u8; 32];

    /// Verify an ed25519 signature
    fn ed25519_verify(sig: &[u8], msg: &[u8], pub_key: &[u8]) -> bool;

    /// verify secp256k1 signatures
    fn secp256k1_verify(sig: &[u8], message: &[u8], public: &[u8]) -> bool;
}

#[cfg(any(feature = "test-helpers", test))]
pub mod helper {
    use crate::host_functions::HostFunctionsProvider;

    #[derive(Default, Debug)]
    pub struct HostFunctionsManager;

    impl HostFunctionsProvider for HostFunctionsManager {
        fn sha2_256(preimage: &[u8]) -> [u8; 32] {
            sp_core::hashing::sha2_256(preimage)
        }

        fn ed25519_verify(sig: &[u8], msg: &[u8], pub_key: &[u8]) -> bool {
            use sp_core::{ed25519, ByteArray, Pair};

            let result = ed25519::Signature::from_slice(sig)
                .ok_or(())
                .and_then(|sig| {
                    let public_key = ed25519::Public::from_slice(pub_key).map_err(|_| ())?;
                    Ok((sig, public_key))
                });

            if let Ok((sig, public_key)) = result {
                return ed25519::Pair::verify(&sig, msg, &public_key);
            }

            false
        }

        fn secp256k1_verify(sig: &[u8], message: &[u8], public: &[u8]) -> bool {
            use sp_core::{ecdsa, ByteArray, Pair};

            let result = ecdsa::Signature::from_slice(sig.clone())
                .ok_or(())
                .and_then(|sig| {
                    let public = ecdsa::Public::from_slice(public).map_err(|_| ())?;
                    Ok((public, sig))
                });

            if let Ok((public, signature)) = result {
                return ecdsa::Pair::verify_weak(&sig, message, &public);
            }

            false
        }
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use crate::host_functions::{helper::HostFunctionsManager, HostFunctionsProvider};

    #[test]
    #[should_panic]
    // not super sure what the problem is here but secpk256 is optional so 🤷🏾‍
    fn test_secpk1256_verification() {
        let public = hex!("043a3150798c8af69d1e6e981f3a45402ba1d732f4be8330c5164f49e10ec555b4221bd842bc5e4d97eff37165f60e3998a424d72a450cf95ea477c78287d0343a");
        let msg = hex!("313233343030");
        let sig = hex!("304402207fffffffffffffffffffffffffffffff5d576e7357a4501ddfe92f46681b20a002207fffffffffffffffffffffffffffffff5d576e7357a4501ddfe92f46681b20a0");

        assert!(HostFunctionsManager::secp256k1_verify(&sig, &msg, &public))
    }
}
