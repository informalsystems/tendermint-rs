//! Provides an interface and default implementation for the `Hasher` operation

use tendermint::Hash;

use crate::types::{Header, ValidatorSet};

/// Hashing for headers and validator sets
pub trait Hasher: Send + Sync {
    /// Hash the given header
    fn hash_header(&self, header: &Header) -> Hash {
        header.hash()
    }

    /// Hash the given validator set
    fn hash_validator_set(&self, validator_set: &ValidatorSet) -> Hash {
        validator_set.hash()
    }
}

/// Default implementation of a hasher
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ProdHasher;

impl Hasher for ProdHasher {}
