//! Provides an interface and default implementation for the `Hasher` operation

use crate::types::{Header, ValidatorSet};

use tendermint::{merkle, Hash};

/// Hashing for headers and validator sets
pub trait Hasher: Send {
    /// Hash the given header
    fn hash_header(&self, header: &Header) -> Hash;

    /// Hash the given validator set
    fn hash_validator_set(&self, validator_set: &ValidatorSet) -> Hash;
}

/// Default implementation of a hasher
#[derive(Clone, Copy, Debug, Default)]
pub struct ProdHasher;

impl Hasher for ProdHasher {
    fn hash_header(&self, header: &Header) -> Hash {
        header.hash()
    }

    /// Compute the Merkle root of the validator set
    fn hash_validator_set(&self, validator_set: &ValidatorSet) -> Hash {
        let validator_bytes: Vec<Vec<u8>> = validator_set
            .validators()
            .iter()
            .map(|validator| validator.hash_bytes())
            .collect();

        Hash::Sha256(merkle::simple_hash_from_byte_vectors(validator_bytes))
    }
}
