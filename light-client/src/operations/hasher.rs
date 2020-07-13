use crate::types::{Header, ValidatorSet};

use tendermint::lite::types::Header as _;
use tendermint::merkle;
use tendermint::Hash;

pub trait Hasher: Send {
    fn hash_header(&self, header: &Header) -> Hash;
    fn hash_validator_set(&self, validator_set: &ValidatorSet) -> Hash;
}

#[derive(Copy, Clone, Debug)]
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
