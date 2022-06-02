//! Host function utilities

/// Host functions that the light client needs for crypto operations.
pub trait HostFunctionsProvider: Send + Sync {
    /// sha256 hash function
    fn sha2_256(preimage: &[u8]) -> [u8; 32];

    /// Verify an ed25519 signature
    fn ed25519_verify(sig: &[u8], msg: &[u8], pub_key: &[u8]) -> bool;
}

#[cfg(test)]
#[derive(Default)]
pub struct TestHostFunctions;

#[cfg(test)]
impl HostFunctionsProvider for TestHostFunctions {
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
}
