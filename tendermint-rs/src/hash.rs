//! Hash functions and their outputs

use algorithm::HashAlgorithm;
use error::Error;
use std::fmt::{self, Display};
use subtle_encoding::{Encoding, Hex};

/// Output size for the SHA-256 hash function
pub const SHA256_HASH_SIZE: usize = 32;

/// Hashes used for computing BlockIDs
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum Hash {
    /// SHA-256 hashes
    Sha256([u8; SHA256_HASH_SIZE]),
}

impl Hash {
    /// Create a new `Hash` with the given algorithm type
    pub fn new(alg: HashAlgorithm, bytes: &[u8]) -> Result<Hash, Error> {
        match alg {
            HashAlgorithm::Sha256 => {
                if bytes.len() == SHA256_HASH_SIZE {
                    let mut h = [0u8; SHA256_HASH_SIZE];
                    h.copy_from_slice(bytes);
                    Ok(Hash::Sha256(h))
                } else {
                    Err(Error::Parse)
                }
            }
        }
    }

    /// Decode a `Hash` from upper-case hexadecimal
    pub fn from_hex_upper(alg: HashAlgorithm, s: &str) -> Result<Hash, Error> {
        match alg {
            HashAlgorithm::Sha256 => {
                let mut h = [0u8; SHA256_HASH_SIZE];
                Hex::upper().decode_to_slice(s.as_bytes(), &mut h)?;
                Ok(Hash::Sha256(h))
            }
        }
    }

    /// Borrow the `Hash` as a byte slice
    pub fn as_slice(&self) -> &[u8] {
        match self {
            Hash::Sha256(ref h) => h.as_ref(),
        }
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let hex = match self {
            Hash::Sha256(ref h) => Hex::upper().encode_to_string(h).unwrap(),
        };

        write!(f, "{}", hex)
    }
}
