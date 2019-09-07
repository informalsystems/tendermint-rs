use byteorder::{ByteOrder, LE};
use ring::aead;

/// Size of a ChaCha20 nonce
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
        let counter: u64 = LE::read_u64(&self.0[4..]);
        LE::write_u64(&mut self.0[4..], counter.checked_add(1).unwrap());
    }

    /// Serialize nonce as bytes (little endian)
    #[inline]
    pub fn to_bytes(&self) -> &[u8] {
        &self.0[..]
    }
}

impl From<&Nonce> for aead::Nonce {
    fn from(nonce: &Nonce) -> aead::Nonce {
        aead::Nonce::assume_unique_for_key(nonce.0)
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
            match check_points.get(&i) {
                Some(want) => {
                    let got = &nonce.to_bytes();
                    assert_eq!(got, want);
                }
                None => (),
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
