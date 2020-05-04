//! Crypto function traits allowing mocking out during testing

use crate::prelude::*;
use anomaly::BoxError;

pub mod header_hasher;
pub use self::header_hasher::*;

pub trait VotingPowerCalculator {
    fn total_power_of(&self, validators: &ValidatorSet) -> u64;
    fn voting_power_in(&self, commit: &Commit, validators: &ValidatorSet) -> u64;
}

impl<T: VotingPowerCalculator> VotingPowerCalculator for &T {
    fn voting_power_in(&self, commit: &Commit, validators: &ValidatorSet) -> u64 {
        (*self).voting_power_in(commit, validators)
    }

    fn total_power_of(&self, validators: &ValidatorSet) -> u64 {
        (*self).total_power_of(validators)
    }
}

impl VotingPowerCalculator for Box<dyn VotingPowerCalculator> {
    fn voting_power_in(&self, commit: &Commit, validators: &ValidatorSet) -> u64 {
        self.as_ref().voting_power_in(commit, validators)
    }

    fn total_power_of(&self, validators: &ValidatorSet) -> u64 {
        self.as_ref().total_power_of(validators)
    }
}

pub trait CommitValidator {
    fn validate(&self, commit: &Commit, validators: &ValidatorSet) -> Result<(), BoxError>;
}

impl<T: CommitValidator> CommitValidator for &T {
    fn validate(&self, commit: &Commit, validators: &ValidatorSet) -> Result<(), BoxError> {
        (*self).validate(commit, validators)
    }
}

impl CommitValidator for Box<dyn CommitValidator> {
    fn validate(&self, commit: &Commit, validators: &ValidatorSet) -> Result<(), BoxError> {
        self.as_ref().validate(commit, validators)
    }
}
