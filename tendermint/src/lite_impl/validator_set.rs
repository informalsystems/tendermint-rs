//! Light client [`ValidatorSet`] implementation for [`Set`].

use crate::validator::Set;
use crate::{lite, merkle, Hash};

impl lite::ValidatorSet for Set {
    /// Compute the Merkle root of the validator set
    fn hash(&self) -> Hash {
        let validator_bytes: Vec<Vec<u8>> = self
            .validators()
            .iter()
            .map(|validator| validator.hash_bytes())
            .collect();
        Hash::Sha256(merkle::simple_hash_from_byte_vectors(validator_bytes))
    }

    fn total_power(&self) -> u64 {
        self.validators().iter().fold(0u64, |total, val_info| {
            total + val_info.voting_power.value()
        })
    }

    fn len(&self) -> usize {
        self.validators().len()
    }

    fn is_empty(&self) -> bool {
        self.validators().is_empty()
    }
}
