use crate::{
    bail,
    predicates::errors::VerificationError,
    types::{SignedHeader, ValidatorSet},
};

use anomaly::BoxError;
use tendermint::lite::types::ValidatorSet as _;

pub trait VotingPowerCalculator: Send {
    fn total_power_of(&self, validators: &ValidatorSet) -> u64;
    fn voting_power_in(
        &self,
        signed_header: &SignedHeader,
        validators: &ValidatorSet,
    ) -> Result<u64, BoxError>;
}

#[derive(Copy, Clone, Debug)]
pub struct ProdVotingPowerCalculator;

impl VotingPowerCalculator for ProdVotingPowerCalculator {
    fn total_power_of(&self, validators: &ValidatorSet) -> u64 {
        validators.total_power()
    }

    fn voting_power_in(
        &self,
        signed_header: &SignedHeader,
        validators: &ValidatorSet,
    ) -> Result<u64, BoxError> {
        // NOTE: We don't know the validators that committed this block,
        //       so we have to check for each vote if its validator is already known.
        let mut signed_power = 0_u64;

        for vote in &signed_header.signed_votes() {
            // Only count if this vote is from a known validator.
            // TODO: we still need to check that we didn't see a vote from this validator twice ...
            let val_id = vote.validator_id();
            let val = match validators.validator(val_id) {
                Some(v) => v,
                None => continue,
            };

            // check vote is valid from validator
            let sign_bytes = vote.sign_bytes();

            if !val.verify_signature(&sign_bytes, vote.signature()) {
                bail!(VerificationError::ImplementationSpecific(format!(
                    "Couldn't verify signature {:?} with validator {:?} on sign_bytes {:?}",
                    vote.signature(),
                    val,
                    sign_bytes,
                )));
            }

            signed_power += val.power();
        }

        Ok(signed_power)
    }
}
