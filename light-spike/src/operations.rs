//! Crypto function traits allowing mocking out during testing

use crate::prelude::*;

pub trait VotingPowerCalculator {
    // TODO: What kind of errors should we be reporting here?
    fn voting_power_in(&self, commit: &Commit, validators: &ValidatorSet) -> Result<u64, Error>;
    fn total_power_of(&self, validators: &ValidatorSet) -> Result<u64, Error>;
}

impl<T: VotingPowerCalculator> VotingPowerCalculator for &T {
    fn voting_power_in(&self, commit: &Commit, validators: &ValidatorSet) -> Result<u64, Error> {
        (*self).voting_power_in(commit, validators)
    }

    fn total_power_of(&self, validators: &ValidatorSet) -> Result<u64, Error> {
        (*self).total_power_of(validators)
    }
}

impl VotingPowerCalculator for Box<dyn VotingPowerCalculator> {
    fn voting_power_in(&self, commit: &Commit, validators: &ValidatorSet) -> Result<u64, Error> {
        self.as_ref().voting_power_in(commit, validators)
    }

    fn total_power_of(&self, validators: &ValidatorSet) -> Result<u64, Error> {
        self.as_ref().total_power_of(validators)
    }
}

pub trait CommitValidator {
    fn validate(&self, commit: &Commit, validators: &ValidatorSet) -> Result<(), Error>;
}

impl<T: CommitValidator> CommitValidator for &T {
    fn validate(&self, commit: &Commit, validators: &ValidatorSet) -> Result<(), Error> {
        (*self).validate(commit, validators)
    }
}

impl CommitValidator for Box<dyn CommitValidator> {
    fn validate(&self, commit: &Commit, validators: &ValidatorSet) -> Result<(), Error> {
        self.as_ref().validate(commit, validators)
    }
}

pub trait HeaderHasher {
    fn hash(&self, header: &Header) -> Hash; // Or Error?
}

impl<T: HeaderHasher> HeaderHasher for &T {
    fn hash(&self, header: &Header) -> Hash {
        (*self).hash(header)
    }
}

impl HeaderHasher for Box<dyn HeaderHasher> {
    fn hash(&self, header: &Header) -> Hash {
        self.as_ref().hash(header)
    }
}
