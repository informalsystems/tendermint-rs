use crate::prelude::*;
use anomaly::BoxError;

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

pub struct ProdCommitValidator;

impl CommitValidator for ProdCommitValidator {
    fn validate(&self, commit: &Commit, validator_set: &ValidatorSet) -> Result<(), BoxError> {
        // TODO: self.commit.block_id cannot be zero in the same way as in go
        // clarify if this another encoding related issue
        if commit.signatures.len() == 0 {
            bail!(VerificationError::ImplementationSpecific(
                "no signatures for commit".to_string()
            ));
        }
        if commit.signatures.len() != validator_set.validators().len() {
            bail!(VerificationError::ImplementationSpecific(format!(
                "pre-commit length: {} doesn't match validator length: {}",
                commit.signatures.len(),
                validator_set.validators().len()
            )));
        }

        for commit_sig in commit.signatures.iter() {
            commit_sig.validate(&validator_set)?;
        }

        Ok(())
    }
}
