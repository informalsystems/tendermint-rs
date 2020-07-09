use std::marker::PhantomData;

use tendermint::lite::types::Header as _;
use tendermint::merkle;
use tendermint::Hash;

use crate::types::{LightBlock, TMHeader, TMLightBlock, TMValidatorSet};

pub trait Hasher<LB: LightBlock>: Send {
    fn hash_header(&self, header: &LB::Header) -> Hash;
    fn hash_validator_set(&self, validator_set: &LB::ValidatorSet) -> Hash;
}

#[derive(Copy, Clone, Debug)]
pub struct ProdHasher<LB> {
    marker: PhantomData<LB>,
}

impl<LB> Default for ProdHasher<LB> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl Hasher<TMLightBlock> for ProdHasher<TMLightBlock> {
    fn hash_header(&self, header: &TMHeader) -> Hash {
        header.hash()
    }

    /// Compute the Merkle root of the validator set
    fn hash_validator_set(&self, validator_set: &TMValidatorSet) -> Hash {
        let validator_bytes: Vec<Vec<u8>> = validator_set
            .validators()
            .iter()
            .map(|validator| validator.hash_bytes())
            .collect();

        Hash::Sha256(merkle::simple_hash_from_byte_vectors(validator_bytes))
    }
}
