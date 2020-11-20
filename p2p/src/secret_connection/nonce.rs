//! Secret Connection nonces

use std::convert::TryInto;

/// Size of a ChaCha20 (IETF) nonce
pub const SIZE: usize = 12;

/// SecretConnection nonces (i.e. ChaCha20 nonces)
pub struct Nonce(pub [u8; SIZE]);

impl Default for Nonce {
    fn default() -> Nonce {
        Nonce([0u8; SIZE])
    }
}

impl Nonce {
    /// Increment the nonce's counter by 1
    pub fn increment(&mut self) {
        let counter: u64 = u64::from_le_bytes(self.0[4..].try_into().unwrap());
        self.0[4..].copy_from_slice(&counter.checked_add(1).unwrap().to_le_bytes());
    }

    /// Serialize nonce as bytes (little endian)
    #[inline]
    pub fn to_bytes(&self) -> &[u8] {
        &self.0[..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_incr_nonce() {
        // make sure we match the golang implementation
        let mut check_points: HashMap<i32, &[u8]> = HashMap::new();
        check_points.insert(0, &[0u8, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0]);
        check_points.insert(1, &[0u8, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0]);
        check_points.insert(510, &[0u8, 0, 0, 0, 255, 1, 0, 0, 0, 0, 0, 0]);
        check_points.insert(511, &[0u8, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0]);
        check_points.insert(512, &[0u8, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0, 0]);
        check_points.insert(1023, &[0u8, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0]);

        let mut nonce = Nonce::default();
        assert_eq!(nonce.to_bytes().len(), SIZE);

        for i in 0..1024 {
            nonce.increment();
            if let Some(want) = check_points.get(&i) {
                let got = &nonce.to_bytes();
                assert_eq!(got, want);
            }
        }
    }
    #[test]
    #[should_panic]
    fn test_incr_nonce_overflow() {
        // other than in the golang implementation we panic if we incremented more than 64
        // bits allow.
        // In golang this would reset to an all zeroes nonce.
        let mut nonce = Nonce([0u8, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255]);
        nonce.increment();
    }
}
